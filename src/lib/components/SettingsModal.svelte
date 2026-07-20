<script lang="ts">
  import Icon from "@iconify/svelte";
  import { untrack } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Modal from "./Modal.svelte";
  import * as api from "$lib/ts/api";
  import type { AppInfo } from "$lib/ts/api";
  import {
    app,
    applyTheme,
    applyAccent,
    applyTextScale,
    applyReduceMotion,
    applyContrast,
    refreshEntries,
    restartAutoSync,
    setError,
    type ThemePreference,
    type TextScale,
  } from "$lib/ts/state.svelte";

  interface Props {
    onclose: () => void;
    onsignin: () => void;
  }

  let { onclose, onsignin }: Props = $props();

  type Section =
    | "files"
    | "appearance"
    | "accessibility"
    | "account"
    | "about";

  const sections: { id: Section; label: string; icon: string }[] = [
    { id: "files", label: "Files", icon: "ph:folder" },
    { id: "appearance", label: "Appearance", icon: "ph:palette" },
    { id: "accessibility", label: "Accessibility", icon: "ph:wheelchair" },
    { id: "account", label: "Account", icon: "ph:user-circle" },
    { id: "about", label: "About", icon: "ph:info" },
  ];

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

  let section = $state<Section>("files");

  let workspaceRoot = $state(untrack(() => app.settings?.workspace_root ?? ""));
  let serverUrl = $state(untrack(() => app.settings?.server_url ?? ""));
  let autosaveSeconds = $state(untrack(() => app.settings?.autosave_seconds ?? 0));
  let syncMinutes = $state(untrack(() => app.settings?.sync_minutes ?? 0));
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
    { value: 1, label: "1 minute" },
    { value: 2, label: "2 minutes" },
    { value: 5, label: "5 minutes" },
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
        syncMinutes,
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
      app.settings = await api.getSettings();
    } catch (error) {
      setError(error);
    }
  }

  const fieldClass =
    "rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none";
</script>

<Modal title="Settings" icon="ph:gear-six" width="max-w-3xl" {onclose}>
  <div class="flex min-h-[360px] gap-5">
    <nav class="flex w-36 shrink-0 flex-col gap-0.5">
      {#each sections as item}
        <button
          class="flex items-center gap-2 rounded-md px-2.5 py-2 text-xs transition
            {section === item.id
            ? 'bg-[var(--color-accent-soft)] font-medium text-[var(--color-accent)]'
            : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)]'}"
          onclick={() => (section = item.id)}
        >
          <Icon icon={item.icon} class="text-base" />
          {item.label}
        </button>
      {/each}
    </nav>

    <div class="min-w-0 flex-1">
      {#if section === "files"}
        <div class="flex flex-col gap-5">
          <div class="flex flex-col gap-1 text-xs">
            <span class="font-medium text-[var(--color-ink-muted)]">
              Workspace folder
            </span>
            <div class="flex gap-2">
              <input class="flex-1 {fieldClass}" bind:value={workspaceRoot} />
              <button
                class="flex items-center gap-1 rounded-md border border-[var(--color-line)] px-3 text-xs hover:bg-[var(--color-surface-muted)]"
                onclick={browse}
              >
                <Icon icon="ph:folder-open" />
                Browse
              </button>
            </div>
            <span class="text-[var(--color-ink-muted)]">
              Projects and documents are stored as plain files here.
            </span>
          </div>

          <div class="flex flex-col gap-1 text-xs">
            <span class="font-medium text-[var(--color-ink-muted)]">Autosave</span>
            <select class={fieldClass} bind:value={autosaveSeconds}>
              {#each autosaveOptions as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
            <span class="text-[var(--color-ink-muted)]">
              Saves the file being edited after you stop typing.
            </span>
          </div>
        </div>
      {:else if section === "appearance"}
        <div class="flex flex-col gap-5">
          <div class="flex flex-col gap-2 text-xs">
            <span class="font-medium text-[var(--color-ink-muted)]">Theme</span>
            <div class="flex gap-2">
              {#each [["light", "Light", "ph:sun"], ["dark", "Dark", "ph:moon"], ["system", "System", "ph:desktop"]] as [value, label, icon]}
                <button
                  class="flex flex-1 items-center justify-center gap-1.5 rounded-md border px-3 py-2.5 transition
                    {app.themePreference === value
                    ? 'border-[var(--color-accent)] bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
                    : 'border-[var(--color-line)] text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)]'}"
                  onclick={() => applyTheme(value as ThemePreference)}
                >
                  <Icon {icon} class="text-base" />
                  {label}
                </button>
              {/each}
            </div>
            <span class="text-[var(--color-ink-muted)]">
              System follows your OS setting and updates live.
            </span>
          </div>

          <div class="flex flex-col gap-2 text-xs">
            <span class="font-medium text-[var(--color-ink-muted)]">
              Accent color
            </span>
            <div class="flex items-center gap-2">
              <button
                class="flex h-7 w-7 items-center justify-center rounded-full border transition
                  {app.accent === null
                  ? 'border-[var(--color-accent)] text-[var(--color-accent)]'
                  : 'border-[var(--color-line)] text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)]'}"
                title="Default"
                onclick={() => applyAccent(null)}
              >
                <Icon icon="ph:arrow-counter-clockwise" class="text-sm" />
              </button>
              {#each accentPresets as preset}
                <button
                  class="h-7 w-7 rounded-full border-2 transition
                    {app.accent === preset
                    ? 'border-[var(--color-ink)]'
                    : 'border-transparent hover:opacity-80'}"
                  style="background-color: {preset}"
                  title={preset}
                  onclick={() => applyAccent(preset)}
                ></button>
              {/each}
              <input
                type="color"
                class="h-7 w-7 cursor-pointer rounded-full border border-[var(--color-line)] bg-transparent p-0"
                value={app.accent ?? "#3b6cf6"}
                title="Custom color"
                oninput={(event) =>
                  applyAccent((event.target as HTMLInputElement).value)}
              />
            </div>
            <span class="text-[var(--color-ink-muted)]">
              Overrides the accent color used across the app.
            </span>
          </div>
        </div>
      {:else if section === "accessibility"}
        <div class="flex flex-col gap-5">
          <div class="flex flex-col gap-2 text-xs">
            <span class="font-medium text-[var(--color-ink-muted)]">
              UI text scale
            </span>
            <div class="flex gap-2">
              {#each textScaleOptions as option}
                <button
                  class="flex flex-1 items-center justify-center rounded-md border px-3 py-2.5 transition
                    {app.textScale === option.value
                    ? 'border-[var(--color-accent)] bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
                    : 'border-[var(--color-line)] text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)]'}"
                  onclick={() => applyTextScale(option.value)}
                >
                  {option.label}
                </button>
              {/each}
            </div>
            <span class="text-[var(--color-ink-muted)]">
              Scales text and controls throughout the app.
            </span>
          </div>

          <div class="flex flex-col gap-2 text-xs">
            <label class="flex items-center gap-2">
              <input
                type="checkbox"
                checked={app.reduceMotion}
                onchange={(event) =>
                  applyReduceMotion(
                    (event.target as HTMLInputElement).checked,
                  )}
              />
              <span class="font-medium text-[var(--color-ink-muted)]">
                Reduce motion
              </span>
            </label>
            <span class="text-[var(--color-ink-muted)]">
              Shortens transitions and animations across the app.
            </span>
          </div>

          <div class="flex flex-col gap-2 text-xs">
            <label class="flex items-center gap-2">
              <input
                type="checkbox"
                checked={app.contrast === "high"}
                onchange={(event) =>
                  applyContrast(
                    (event.target as HTMLInputElement).checked
                      ? "high"
                      : "normal",
                  )}
              />
              <span class="font-medium text-[var(--color-ink-muted)]">
                High contrast
              </span>
            </label>
            <span class="text-[var(--color-ink-muted)]">
              Increases contrast for borders, muted text, and focus outlines.
            </span>
          </div>
        </div>
      {:else if section === "account"}
        <div class="flex flex-col gap-5">
          <div
            class="flex items-center gap-3 rounded-md border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-3"
          >
            <Icon
              icon={app.account ? "ph:user-circle-check" : "ph:user-circle"}
              class="text-2xl {app.account
                ? 'text-[var(--color-success)]'
                : 'text-[var(--color-ink-muted)]'}"
            />
            <div class="flex-1 text-xs">
              {#if app.account}
                <p class="font-medium">{app.account.username}</p>
                <p class="text-[var(--color-ink-muted)]">{app.account.email}</p>
              {:else}
                <p class="font-medium">Not connected</p>
                <p class="text-[var(--color-ink-muted)]">
                  Sign in to sync projects to the cloud.
                </p>
              {/if}
            </div>
            {#if app.account}
              <button
                class="rounded-md border border-[var(--color-line)] px-3 py-1.5 text-xs hover:bg-[var(--color-surface)]"
                onclick={signOut}
              >
                Sign out
              </button>
            {:else}
              <button
                class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white hover:opacity-90"
                onclick={onsignin}
              >
                Sign in
              </button>
            {/if}
          </div>

          <div class="flex flex-col gap-1 text-xs">
            <span class="font-medium text-[var(--color-ink-muted)]">
              TypstDrive server
            </span>
            <input class={fieldClass} bind:value={serverUrl} />
          </div>

          <div class="flex flex-col gap-1 text-xs">
            <span class="font-medium text-[var(--color-ink-muted)]">
              Automatic sync
            </span>
            <select class={fieldClass} bind:value={syncMinutes}>
              {#each syncOptions as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
            <span class="text-[var(--color-ink-muted)]">
              Pulls and pushes cloud-linked projects on a timer. Conflicts pause
              syncing until they are resolved.
            </span>
          </div>
        </div>
      {:else}
        <div class="flex flex-col gap-4">
          <div class="flex items-center gap-3">
            <Icon icon="ph:file-code" class="text-3xl text-[var(--color-accent)]" />
            <div>
              <p class="text-sm font-semibold">Typst Desktop</p>
              <p class="text-xs text-[var(--color-ink-muted)]">
                A local editor for Typst documents
              </p>
            </div>
          </div>

          {#if info}
            <dl class="flex flex-col text-xs">
              {#each [["Version", info.version], ["Typst", info.typst_version], ["Tauri", info.tauri_version], ["Author", info.authors], ["License", info.license]] as [label, value]}
                <div
                  class="flex justify-between border-b border-[var(--color-line)] py-1.5 last:border-0"
                >
                  <dt class="text-[var(--color-ink-muted)]">{label}</dt>
                  <dd class="font-medium">{value}</dd>
                </div>
              {/each}
            </dl>
          {/if}

          <div class="flex flex-col gap-1.5">
            {#each links as link}
              <button
                class="flex items-center gap-2 rounded-md border border-[var(--color-line)] px-3 py-2 text-xs transition hover:border-[var(--color-accent)] hover:bg-[var(--color-surface-muted)]"
                onclick={() => openUrl(link.url)}
              >
                <Icon icon={link.icon} class="text-base text-[var(--color-accent)]" />
                <span class="flex-1 text-left">{link.label}</span>
                <Icon
                  icon="ph:arrow-square-out"
                  class="text-[var(--color-ink-muted)]"
                />
              </button>
            {/each}
          </div>

          <p
            class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-3 text-[11px] leading-relaxed text-[var(--color-ink-muted)]"
          >
            Typst Desktop is a community application and is not affiliated with,
            endorsed by, or supported by the official Typst project. The links
            above open the official Typst website in your browser.
          </p>
        </div>
      {/if}
    </div>
  </div>

  {#snippet footer()}
    {@const draftless =
      section === "about" ||
      section === "appearance" ||
      section === "accessibility"}
    <button
      class="rounded-md px-3 py-1.5 text-xs text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]"
      onclick={onclose}
    >
      {draftless ? "Close" : "Cancel"}
    </button>
    {#if !draftless}
      <button
        class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90 disabled:opacity-50"
        disabled={saving}
        onclick={save}
      >
        {saving ? "Saving..." : "Save"}
      </button>
    {/if}
  {/snippet}
</Modal>
