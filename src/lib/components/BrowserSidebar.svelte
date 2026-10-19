<script lang="ts">
  import Icon from "@iconify/svelte";
  import { app, browseTo, openCloudFolder } from "$lib/ts/state.svelte";

  interface Props {
    onsettings: () => void;
  }

  let { onsettings }: Props = $props();

  const COLLAPSED_KEY = "typst-desktop-browser-sidebar-collapsed";

  function readCollapsed() {
    try {
      return localStorage.getItem(COLLAPSED_KEY) === "true";
    } catch {
      return false;
    }
  }

  let collapsed = $state(readCollapsed());

  function toggleCollapsed() {
    collapsed = !collapsed;
    try {
      localStorage.setItem(COLLAPSED_KEY, String(collapsed));
    } catch {}
  }

  function openWorkspace() {
    app.scope = "local";
    browseTo("");
  }

  function openCloud(folder: null | "shared") {
    app.scope = "cloud";
    openCloudFolder(folder);
  }

  const inWorkspace = $derived(app.scope === "local");
  const inCloudHome = $derived(app.scope === "cloud" && app.cloudFolder !== "shared");
  const inShared = $derived(app.scope === "cloud" && app.cloudFolder === "shared");
  const accountInitial = $derived(app.account?.username?.charAt(0).toUpperCase() ?? "?");
</script>

{#snippet navItem(label: string, icon: string, active: boolean, onclick: () => void)}
  <button
    {onclick}
    title={collapsed ? label : undefined}
    aria-label={label}
    class="flex w-full items-center gap-2.5 rounded-md py-1.5 text-xs transition-colors
      {collapsed ? 'justify-center px-0' : 'px-2.5'}
      {active
      ? 'bg-[var(--color-surface-sunken)] font-semibold text-[var(--color-ink)]'
      : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)]'}"
  >
    <Icon {icon} class="shrink-0 text-base {active ? 'text-[var(--color-accent)]' : ''}" />
    {#if !collapsed}{label}{/if}
  </button>
{/snippet}

{#snippet groupLabel(label: string)}
  {#if collapsed}
    <div class="mx-2 mb-1 h-px bg-[var(--color-line)]"></div>
  {:else}
    <p class="px-2.5 pb-1 text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">
      {label}
    </p>
  {/if}
{/snippet}

<aside
  class="flex shrink-0 flex-col overflow-y-auto overflow-x-hidden border-r border-[var(--color-line)] bg-[var(--color-surface)] transition-[width] duration-200
    {collapsed ? 'w-12' : 'w-52'}"
>
  <nav class="flex-1 space-y-4 {collapsed ? 'p-1.5' : 'p-2.5'}">
    <div class="space-y-0.5">
      {@render groupLabel("This computer")}
      {@render navItem("Workspace", "ph:hard-drives", inWorkspace, openWorkspace)}
    </div>
    {#if app.account}
      <div class="space-y-0.5">
        {@render groupLabel("TypstDrive")}
        {@render navItem("Home", "ph:cloud", inCloudHome, () => openCloud(null))}
        {@render navItem("Shared", "ph:users-three", inShared, () => openCloud("shared"))}
      </div>
    {/if}
  </nav>

  <div class="space-y-1 border-t border-[var(--color-line)] {collapsed ? 'p-1.5' : 'p-2.5'}">
    {#if app.account}
      <div
        class="flex items-center gap-2 py-1 {collapsed ? 'justify-center' : 'px-1'}"
        title={collapsed ? `${app.account.username} (${app.account.email})` : undefined}
      >
        <span class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-[var(--color-accent-soft)] text-xs font-bold text-[var(--color-accent)]">
          {accountInitial}
        </span>
        {#if !collapsed}
          <span class="min-w-0">
            <span class="block truncate text-xs font-semibold">{app.account.username}</span>
            <span class="block truncate text-[10px] text-[var(--color-ink-muted)]">{app.account.email}</span>
          </span>
        {/if}
      </div>
    {/if}
    {@render navItem("Settings", "ph:gear-six", false, onsettings)}
    {@render navItem(collapsed ? "Expand sidebar" : "Collapse", collapsed ? "ph:caret-double-right" : "ph:caret-double-left", false, toggleCollapsed)}
  </div>
</aside>
