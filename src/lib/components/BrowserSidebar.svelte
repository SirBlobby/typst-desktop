<script lang="ts">
  import Icon from "@iconify/svelte";
  import { app, browseTo } from "#lib/ts/state.svelte";
  import { openCloudSpace } from "#lib/ts/cloud-spaces";
  import AccountAvatar from "./AccountAvatar.svelte";
  import SidebarShortcuts from "./SidebarShortcuts.svelte";

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

  const inWorkspace = $derived(app.scope === "local" && !app.currentDir.startsWith("shortcut:"));
  const inPersonalCloud = $derived(app.scope === "cloud" && app.cloudSpace === null);
  const inCloudHome = $derived(inPersonalCloud && app.cloudFolder !== "shared");
  const inShared = $derived(inPersonalCloud && app.cloudFolder === "shared");

  function inTeam(teamId: string) {
    return app.scope === "cloud" && app.cloudSpace === teamId;
  }
</script>

{#snippet navItem(label: string, icon: string, active: boolean, onclick: () => void)}
  <button
    {onclick}
    title={collapsed ? label : undefined}
    aria-label={label}
    class="flex h-8 w-full items-center gap-2.5 rounded-md text-[13px] transition-colors
      {collapsed ? 'justify-center px-0' : 'px-2.5'}
      {active
      ? 'bg-[var(--color-surface-sunken)] font-medium text-[var(--color-ink)]'
      : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]/60 hover:text-[var(--color-ink)]'}"
  >
    <Icon {icon} class="shrink-0 text-base {active ? 'text-[var(--color-accent)]' : ''}" />
    {#if !collapsed}{label}{/if}
  </button>
{/snippet}

{#snippet groupLabel(label: string)}
  {#if collapsed}
    <div class="mx-2 mb-1 h-px bg-[var(--color-line)]"></div>
  {:else}
    <p class="section-label px-2.5 pb-1.5">
      {label}
    </p>
  {/if}
{/snippet}

<aside
  class="flex shrink-0 flex-col overflow-y-auto overflow-x-hidden border-r border-[var(--color-line)] bg-[var(--color-surface)] transition-[width] duration-200
    {collapsed ? 'w-12' : 'w-56'}"
>
  <nav class="flex-1 space-y-5 {collapsed ? 'p-1.5' : 'px-2.5 pb-2.5 pt-4'}">
    <div class="space-y-0.5">
      {@render groupLabel("This computer")}
      {@render navItem("Workspace", "ph:hard-drives", inWorkspace, openWorkspace)}
      <SidebarShortcuts {collapsed} />
    </div>
    {#if app.account}
      <div class="space-y-0.5">
        {@render groupLabel("TypstDrive")}
        {@render navItem("Home", "ph:cloud", inCloudHome, () => openCloudSpace(null))}
        {@render navItem("Shared", "ph:users-three", inShared, () => openCloudSpace(null, "shared"))}
      </div>
      {#if app.cloudSpaces.length > 0}
        <div class="space-y-0.5">
          {@render groupLabel("Team spaces")}
          {#each app.cloudSpaces as space (space.id)}
            {@render navItem(space.name, "ph:users-four", inTeam(space.id), () => openCloudSpace(space.id))}
          {/each}
        </div>
      {/if}
    {/if}
  </nav>

  <div class="space-y-1 border-t border-[var(--color-line)] {collapsed ? 'p-1.5' : 'p-2.5'}">
    {#if app.account}
      <div
        class="flex items-center gap-2 py-1 {collapsed ? 'justify-center' : 'px-1'}"
        title={collapsed ? `${app.account.username} (${app.account.email})` : undefined}
      >
        <AccountAvatar />
        {#if !collapsed}
          <span class="min-w-0">
            <span class="block truncate text-xs font-medium">{app.account.username}</span>
            <span class="block truncate text-[11px] text-[var(--color-ink-muted)]">{app.account.email}</span>
          </span>
        {/if}
      </div>
    {/if}
    {@render navItem("Settings", "ph:gear-six", false, onsettings)}
    {@render navItem(collapsed ? "Expand sidebar" : "Collapse", collapsed ? "ph:caret-double-right" : "ph:caret-double-left", false, toggleCollapsed)}
  </div>
</aside>
