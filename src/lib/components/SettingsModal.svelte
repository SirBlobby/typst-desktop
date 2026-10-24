<script lang="ts">
  import Icon from "@iconify/svelte";
  import { untrack } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Modal from "./Modal.svelte";
  import SettingsHeader from "./settings/SettingsHeader.svelte";
  import SettingsCard from "./settings/SettingsCard.svelte";
  import SettingsRow from "./settings/SettingsRow.svelte";
  import ToggleSwitch from "./settings/ToggleSwitch.svelte";
  import EditorSection from "./settings/EditorSection.svelte";
  import CitationsSection from "./settings/CitationsSection.svelte";
  import AccountAvatar from "./AccountAvatar.svelte";
  import * as api from "$lib/ts/api";
  import type { AppInfo, CompatibilityStatus } from "$lib/ts/api";
  import {
    HOTKEY_DEFS,
    comboFromEvent,
    isCustomized,
    keysFor,
    rebindHotkey,
    resetHotkey,
  } from "$lib/ts/hotkeys";
  import {
    app,
    applyTheme,
    applyColorTheme,
    applyAccent,
    applyTextScale,
    applyReduceMotion,
    applyContrast,
    colorThemes,
    refreshEntries,
    restartAutoSync,
    setError,
    type ThemePreference,
    type TextScale,
  } from "$lib/ts/state.svelte";
  import { clearCloudSpaces } from "$lib/ts/cloud-spaces";

  interface Props {
    initialSection?: Section;
    onclose: () => void;
    onsignin: () => void;
  }

  let { initialSection = "files", onclose, onsignin }: Props = $props();

  type Section =
    | "files"
    | "appearance"
    | "editor"
    | "accessibility"
    | "account"
    | "citations"
    | "lsp"
    | "hotkeys"
    | "about";

  const sections: { id: Section; label: string; icon: string }[] = [
    { id: "files", label: "Files", icon: "ph:folder" },
    { id: "appearance", label: "Appearance", icon: "ph:palette" },
    { id: "editor", label: "Editor", icon: "ph:code" },
    { id: "accessibility", label: "Accessibility", icon: "ph:wheelchair" },
    { id: "account", label: "Account", icon: "ph:user-circle" },
    { id: "citations", label: "Citations", icon: "ph:quotes" },
    { id: "lsp", label: "Language Server", icon: "ph:plugs-connected" },
    { id: "hotkeys", label: "Hotkeys", icon: "ph:keyboard" },
    { id: "about", label: "About", icon: "ph:info" },
  ];

  const isMac =
    typeof navigator !== "undefined" &&
    /mac/i.test(navigator.platform ?? navigator.userAgent);

  function formatKeys(combo: string): string[] {
    const variants = combo.split(",");
    const preferred =
      variants.find((variant) =>
        isMac ? variant.includes("command") : !variant.includes("command"),
      ) ?? variants[0];

    return preferred.split("+").map((part) => {
      switch (part) {
        case "command":
          return "⌘";
        case "ctrl":
          return "Ctrl";
        case "alt":
          return isMac ? "⌥" : "Alt";
        case "shift":
          return "Shift";
        case "esc":
          return "Esc";
        case "space":
          return "Space";
        case "up":
          return "↑";
        case "down":
          return "↓";
        case "left":
          return "←";
        case "right":
          return "→";
        default:
          return part.length === 1 ? part.toUpperCase() : part;
      }
    });
  }

  let editingId = $state<string | null>(null);
  let hotkeyVersion = $state(0);

  const editableHotkeyGroups = $derived.by(() => {
    hotkeyVersion;
    const groups = new Map<string, typeof HOTKEY_DEFS>();
    for (const def of HOTKEY_DEFS) {
      if (!groups.has(def.group)) groups.set(def.group, []);
      groups.get(def.group)!.push(def);
    }
    return Array.from(groups.entries()).map(([title, defs]) => ({
      title,
      items: defs.map((def) => ({
        id: def.id,
        label: def.label,
        keys: keysFor(def.id),
        customized: isCustomized(def.id),
      })),
    }));
  });

  $effect(() => {
    if (!editingId) return;
    const id = editingId;

    function handleCapture(event: KeyboardEvent) {
      event.preventDefault();
      event.stopPropagation();
      if (event.key === "Escape") {
        editingId = null;
        return;
      }
      const combo = comboFromEvent(event);
      if (!combo) return;
      rebindHotkey(id, combo);
      hotkeyVersion++;
      editingId = null;
    }

    window.addEventListener("keydown", handleCapture, true);
    return () => window.removeEventListener("keydown", handleCapture, true);
  });

  const staticHotkeyGroups: { title: string; items: { keys: string[]; label: string }[] }[] = [
    {
      title: "File browser",
      items: [
        { keys: ["F2"], label: "Rename selected entry" },
        { keys: ["Delete"], label: "Delete selected entries" },
        { keys: ["Esc"], label: "Cancel rename or clear selection" },
      ],
    },
    {
      title: "Image viewer",
      items: [
        { keys: ["←"], label: "Previous image" },
        { keys: ["→"], label: "Next image" },
        { keys: ["Esc"], label: "Close viewer" },
      ],
    },
    {
      title: "General",
      items: [{ keys: ["Esc"], label: "Close dialog" }],
    },
  ];

  const lspLabel: Record<string, string> = {
    off: "Off",
    starting: "Starting…",
    on: "Running",
    unavailable: "Unavailable",
  };

  const accentPresets = [
    "#3b6cf6",
    "#7c5cfc",
    "#22b573",
    "#f2994a",
    "#ec4899",
    "#14b8a6",
  ];

  const textScaleOptions: { value: TextScale; label: string }[] = [
    { value: "small", label: "Small" },
    { value: "default", label: "Default" },
    { value: "large", label: "Large" },
    { value: "xlarge", label: "Extra large" },
  ];

  let section = $state<Section>(untrack(() => initialSection));

  let workspaceRoot = $state(untrack(() => app.settings?.workspace_root ?? ""));
  let serverUrl = $state(untrack(() => app.settings?.server_url ?? ""));
  let compatibility = $state<CompatibilityStatus | null>(null);
  let checkingCompatibility = $state(false);
  let compatibilityTimer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    const url = serverUrl.trim();
    clearTimeout(compatibilityTimer);
    if (!url) {
      compatibility = null;
      return;
    }
    compatibilityTimer = setTimeout(async () => {
      checkingCompatibility = true;
      try {
        compatibility = await api.cloudCheckCompatibility(url);
      } catch {
        compatibility = null;
      } finally {
        checkingCompatibility = false;
      }
    }, 600);
  });
  let autosaveSeconds = $state(untrack(() => app.settings?.autosave_seconds ?? 0));
  let syncSeconds = $state(untrack(() => app.settings?.sync_seconds ?? 0));
  let saving = $state(false);
  let info = $state<AppInfo | null>(null);

  $effect(() => {
    if (section === "about" && !info) {
      api
        .appInfo()
        .then((result) => (info = result))
        .catch(() => (info = null));
    }
  });

  const autosaveOptions = [
    { value: 0, label: "Off" },
    { value: 5, label: "5 seconds" },
    { value: 10, label: "10 seconds" },
    { value: 15, label: "15 seconds" },
  ];

  const syncOptions = [
    { value: 0, label: "Off" },
    { value: 15, label: "15 seconds" },
    { value: 30, label: "30 seconds" },
    { value: 60, label: "1 minute" },
    { value: 120, label: "2 minutes" },
    { value: 300, label: "5 minutes" },
  ];

  const links = [
    {
      label: "Typst Documentation",
      url: "https://typst.app/docs/",
      icon: "ph:book-open",
    },
    {
      label: "Typst Universe",
      url: "https://typst.app/universe/",
      icon: "ph:planet",
    },
    {
      label: "GitHub Repository",
      url: "https://github.com/SirBlobby/typst-desktop.git",
      icon: "ph:github-logo",
    },
    {
      label: "Report an Issue",
      url: "https://github.com/SirBlobby/typst-desktop/issues",
      icon: "ph:bug",
    },
  ];

  async function browse() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      workspaceRoot = selected;
    }
  }

  async function save() {
    saving = true;
    try {
      app.settings = await api.updateSettings({
        workspaceRoot,
        serverUrl,
        autosaveSeconds,
        syncSeconds,
      });
      restartAutoSync();
      await refreshEntries();
      onclose();
    } catch (error) {
      setError(error);
    } finally {
      saving = false;
    }
  }

  async function signOut() {
    try {
      await api.cloudLogout();
      app.account = null;
      app.cloudProjects = [];
      await clearCloudSpaces();
      app.scope = "local";
      app.settings = await api.getSettings();
    } catch (error) {
      setError(error);
    }
  }

  const fieldClass = "field field-sm h-8 w-auto";
</script>

{#snippet choiceButton(active: boolean, onclick: () => void, label: string, icon?: string, swatch?: string)}
  <button
    class="flex h-7 items-center gap-1.5 rounded-md px-2.5 text-xs transition
      {active
      ? 'bg-[var(--color-surface-raised)] font-medium text-[var(--color-ink)] shadow-sm'
      : 'text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
    {onclick}
  >
    {#if swatch}
      <span class="h-2.5 w-2.5 rounded-full" style="background-color: {swatch}"></span>
    {/if}
    {#if icon}
      <Icon {icon} class="text-sm" />
    {/if}
    {label}
  </button>
{/snippet}

{#snippet hotkeyGroup(title: string, items: { id?: string; label: string; keys: string | string[]; customized?: boolean }[], editable: boolean)}
  <SettingsCard {title}>
    {#each items as item}
      <div class="flex items-center justify-between gap-4 px-4 py-2">
        <span class="text-xs">{item.label}</span>
        {#if editable && editingId === item.id}
          <span class="text-xs text-[var(--color-accent)]">Press keys… (Esc to cancel)</span>
        {:else}
          <span class="flex shrink-0 items-center gap-1">
            {#each typeof item.keys === "string" ? formatKeys(item.keys) : item.keys as key}
              <kbd class="rounded-md border border-b-2 border-[var(--color-line)] bg-[var(--color-surface-muted)] px-1.5 py-0.5 font-sans text-[10px] font-medium text-[var(--color-ink-muted)]">
                {key}
              </kbd>
            {/each}
            {#if editable && item.customized}
              <button
                class="icon-btn icon-btn-sm"
                title="Reset to default"
                aria-label="Reset to default"
                onclick={() => {
                  resetHotkey(item.id!);
                  hotkeyVersion++;
                }}
              >
                <Icon icon="ph:arrow-counter-clockwise" class="text-xs" />
              </button>
            {/if}
            {#if editable}
              <button
                class="icon-btn icon-btn-sm"
                title="Change shortcut"
                aria-label="Change shortcut"
                onclick={() => (editingId = item.id!)}
              >
                <Icon icon="ph:pencil-simple" class="text-xs" />
              </button>
            {/if}
          </span>
        {/if}
      </div>
    {/each}
  </SettingsCard>
{/snippet}

<Modal title="Settings" icon="ph:gear-six" width="max-w-4xl" {onclose}>
  <div class="-mx-5 -mb-5 -mt-3 flex h-[560px] max-h-[72vh] border-t border-[var(--color-line)]">
    <nav class="flex w-48 shrink-0 flex-col gap-0.5 border-r border-[var(--color-line)] bg-[var(--color-surface-raised)] p-2">
      {#each sections as item}
        <button
          class="flex h-8 items-center gap-2 rounded-md px-2.5 text-[13px] transition
            {section === item.id
            ? 'bg-[var(--color-accent-soft)] font-medium text-[var(--color-ink)]'
            : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]'}"
          onclick={() => (section = item.id)}
        >
          <Icon icon={item.icon} class="text-base {section === item.id ? 'text-[var(--color-accent)]' : ''}" />
          {item.label}
        </button>
      {/each}
    </nav>

    <div class="scroll-thin min-w-0 flex-1 overflow-y-auto bg-[var(--color-surface-muted)] px-6 py-5">
      {#if section === "files"}
        <SettingsHeader title="Files" description="Where Typst Desktop keeps your work and when it saves." icon="ph:folder" />
        <div class="flex flex-col gap-4">
          <SettingsCard>
            <SettingsRow label="Workspace folder" description="Projects and documents are stored as plain files here." stacked>
              <input class="{fieldClass} flex-1" bind:value={workspaceRoot} />
              <button
                class="btn btn-secondary h-8"
                onclick={browse}
              >
                <Icon icon="ph:folder-open" />
                Browse
              </button>
            </SettingsRow>
            <SettingsRow
              label="Autosave"
              description="Saves the open file after you stop typing. Cloud-linked files always save every few seconds because they sync live."
            >
              <select class={fieldClass} bind:value={autosaveSeconds}>
                {#each autosaveOptions as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </SettingsRow>
          </SettingsCard>
        </div>
      {:else if section === "appearance"}
        <SettingsHeader title="Appearance" description="The same themes as TypstDrive, in light or dark." icon="ph:palette" />
        <div class="flex flex-col gap-4">
          <SettingsCard title="Theme">
            <SettingsRow label="Mode" description="System follows your operating system and updates live.">
              <div class="flex rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-sunken)] p-0.5">
                {#each [["light", "Light", "ph:sun"], ["dark", "Dark", "ph:moon"], ["system", "System", "ph:desktop"]] as [value, label, icon]}
                  {@render choiceButton(app.themePreference === value, () => applyTheme(value as ThemePreference), label, icon)}
                {/each}
              </div>
            </SettingsRow>
            <SettingsRow label="Color theme" description="Sets the surfaces, editor colors and default accent.">
              <div class="flex rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-sunken)] p-0.5">
                {#each colorThemes as entry}
                  {@render choiceButton(app.colorTheme === entry.id, () => applyColorTheme(entry.id), entry.label, entry.icon)}
                {/each}
              </div>
            </SettingsRow>
            <SettingsRow label="Accent color" description="Overrides the theme's accent color.">
              <button
                class="flex h-6 w-6 items-center justify-center rounded-full border transition
                  {app.accent === null
                  ? 'border-[var(--color-accent)] text-[var(--color-accent)]'
                  : 'border-[var(--color-line)] text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]'}"
                title="Use the theme accent"
                aria-label="Use the theme accent"
                onclick={() => applyAccent(null)}
              >
                <Icon icon="ph:arrow-counter-clockwise" class="text-xs" />
              </button>
              {#each accentPresets as preset}
                <button
                  class="h-6 w-6 rounded-full border-2 transition {app.accent === preset ? 'border-[var(--color-ink)]' : 'border-transparent hover:opacity-80'}"
                  style="background-color: {preset}"
                  title={preset}
                  aria-label={`Accent ${preset}`}
                  onclick={() => applyAccent(preset)}
                ></button>
              {/each}
              <input
                type="color"
                class="h-6 w-6 cursor-pointer rounded-full border border-[var(--color-line)] bg-transparent p-0"
                value={app.accent ?? "#1e66f5"}
                title="Custom color"
                oninput={(event) => applyAccent((event.target as HTMLInputElement).value)}
              />
            </SettingsRow>
          </SettingsCard>
        </div>
      {:else if section === "editor"}
        <EditorSection {fieldClass} />
      {:else if section === "accessibility"}
        <SettingsHeader title="Accessibility" description="Make the app easier to read and calmer to use." icon="ph:wheelchair" />
        <SettingsCard>
          <SettingsRow label="Text size" description="Scales text and controls throughout the app.">
            <div class="flex rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-sunken)] p-0.5">
              {#each textScaleOptions as option}
                {@render choiceButton(app.textScale === option.value, () => applyTextScale(option.value), option.label)}
              {/each}
            </div>
          </SettingsRow>
          <SettingsRow label="Reduce motion" description="Shortens transitions and animations.">
            <ToggleSwitch label="Reduce motion" checked={app.reduceMotion} onchange={applyReduceMotion} />
          </SettingsRow>
          <SettingsRow label="High contrast" description="Stronger text, borders, backgrounds and focus outlines.">
            <ToggleSwitch label="High contrast" checked={app.contrast === "high"} onchange={(value) => applyContrast(value ? "high" : "normal")} />
          </SettingsRow>
        </SettingsCard>
      {:else if section === "account"}
        <SettingsHeader title="Account" description="Connect to a TypstDrive server to sync projects and documents." icon="ph:user-circle" />
        <div class="flex flex-col gap-4">
          <SettingsCard>
            <div class="flex items-center gap-3 px-4 py-3">
              <AccountAvatar sizeClass="h-9 w-9 text-sm" />
              <div class="min-w-0 flex-1 text-xs">
                {#if app.account}
                  <p class="truncate font-semibold">{app.account.username}</p>
                  <p class="truncate text-[var(--color-ink-muted)]">{app.account.email}</p>
                {:else}
                  <p class="font-semibold">Not connected</p>
                  <p class="text-[var(--color-ink-muted)]">Sign in to sync projects to the cloud.</p>
                {/if}
              </div>
              {#if app.account}
                <button class="btn btn-secondary" onclick={signOut}>
                  Sign out
                </button>
              {:else}
                <button class="btn btn-primary" onclick={onsignin}>
                  Sign in
                </button>
              {/if}
            </div>
          </SettingsCard>

          <SettingsCard title="Server">
            <SettingsRow label="TypstDrive server" description="The address of your TypstDrive instance." stacked>
              <div class="flex w-full flex-col gap-1">
                <input class="{fieldClass} w-full" bind:value={serverUrl} />
                {#if checkingCompatibility}
                  <span class="flex items-center gap-1 text-[11px] text-[var(--color-ink-muted)]">
                    <Icon icon="ph:circle-notch" class="animate-spin" />
                    Checking server version...
                  </span>
                {:else if compatibility && !compatibility.compatible}
                  <span class="flex items-center gap-1 text-[11px] text-[var(--color-danger)]">
                    <Icon icon="ph:warning-circle" />
                    {compatibility.message}
                  </span>
                {:else if compatibility && compatibility.compatible}
                  <span class="flex items-center gap-1 text-[11px] text-[var(--color-success)]">
                    <Icon icon="ph:check-circle" />
                    Compatible (server v{compatibility.server_version})
                  </span>
                {/if}
              </div>
            </SettingsRow>
            <SettingsRow
              label="Automatic sync"
              description="Pulls and pushes cloud-linked projects on a timer when live sync is unavailable. Conflicts pause syncing until resolved."
            >
              <select class={fieldClass} bind:value={syncSeconds}>
                {#each syncOptions as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </SettingsRow>
          </SettingsCard>
        </div>
      {:else if section === "citations"}
        <CitationsSection {fieldClass} />
      {:else if section === "lsp"}
        <SettingsHeader title="Language Server" description="Autocomplete, diagnostics and hover help from tinymist." icon="ph:plugs-connected" />
        <div class="flex flex-col gap-4">
          <SettingsCard>
            <div class="flex items-center gap-3 px-4 py-3">
              <span
                class="h-2.5 w-2.5 rounded-full
                  {app.lspStatus === 'on'
                  ? 'bg-[var(--color-success)]'
                  : app.lspStatus === 'starting'
                    ? 'bg-[var(--color-accent)]'
                    : 'bg-[var(--color-ink-muted)]'}"
              ></span>
              <div class="text-xs">
                <p class="font-semibold">{lspLabel[app.lspStatus]}</p>
                <p class="text-[var(--color-ink-muted)]">Reflects the file currently open in the editor.</p>
              </div>
            </div>
            <p class="px-4 py-3 text-xs text-[var(--color-ink-muted)]">
              Typst Desktop uses <span class="font-medium text-[var(--color-ink)]">tinymist</span>, the official Typst
              language server. It must be installed and available on your PATH; Typst Desktop does not bundle it.
            </p>
          </SettingsCard>

          {#if app.lspStatus === "unavailable"}
            <div class="rounded-lg border border-[var(--color-danger)]/25 bg-[var(--color-danger)]/8 p-3 text-xs text-[var(--color-danger)]">
              tinymist could not be started. Confirm it's installed and on your PATH, then reopen the file.
            </div>
          {/if}
        </div>
      {:else if section === "hotkeys"}
        <SettingsHeader title="Hotkeys" description="Click the pencil next to a shortcut and press a new key combination. Press Esc to cancel." icon="ph:keyboard" />
        <div class="flex flex-col gap-4">
          {#each editableHotkeyGroups as group}
            {@render hotkeyGroup(group.title, group.items, true)}
          {/each}
          {#each staticHotkeyGroups as group}
            {@render hotkeyGroup(group.title, group.items, false)}
          {/each}
        </div>
      {:else}
        <SettingsHeader title="About" description="Version information and links." icon="ph:info" />
        <div class="flex flex-col gap-4">
          <SettingsCard>
            <div class="flex items-center gap-3 px-4 py-3">
              <img src="/favicon.png" alt="Typst Desktop" class="h-11 w-11 rounded-xl" />
              <div>
                <p class="text-sm font-semibold">Typst Desktop</p>
                <p class="text-xs text-[var(--color-ink-muted)]">A local editor for Typst documents</p>
              </div>
            </div>
            {#if info}
              {#each [["Version", info.version], ["Typst", info.typst_version], ["Tauri", info.tauri_version], ["Author", info.authors], ["License", info.license]] as [label, value]}
                <div class="flex justify-between px-4 py-2 text-xs">
                  <span class="text-[var(--color-ink-muted)]">{label}</span>
                  <span class="font-medium">{value}</span>
                </div>
              {/each}
            {/if}
          </SettingsCard>

          <SettingsCard title="Links">
            {#each links as link}
              <button class="flex w-full items-center gap-2 px-4 py-2.5 text-xs transition hover:bg-[var(--color-surface-sunken)]/60" onclick={() => openUrl(link.url)}>
                <Icon icon={link.icon} class="text-base text-[var(--color-accent)]" />
                <span class="flex-1 text-left">{link.label}</span>
                <Icon icon="ph:arrow-square-out" class="text-[var(--color-ink-muted)]" />
              </button>
            {/each}
          </SettingsCard>

          <p class="text-[11px] leading-relaxed text-[var(--color-ink-muted)]">
            Typst Desktop is a community application and is not affiliated with, endorsed by, or supported by the
            official Typst project. The links above open the official Typst website in your browser.
          </p>
        </div>
      {/if}
    </div>
  </div>

  {#snippet footer()}
    {@const draftless =
      section === "about" ||
      section === "appearance" ||
      section === "editor" ||
      section === "accessibility" ||
      section === "citations" ||
      section === "lsp" ||
      section === "hotkeys"}
    <button
      class="btn btn-secondary"
      onclick={onclose}
    >
      {draftless ? "Close" : "Cancel"}
    </button>
    {#if !draftless}
      <button
        class="btn btn-primary"
        disabled={saving}
        onclick={save}
      >
        {saving ? "Saving..." : "Save"}
      </button>
    {/if}
  {/snippet}
</Modal>
