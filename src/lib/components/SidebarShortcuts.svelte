<script lang="ts">
  import Icon from "@iconify/svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import * as api from "$lib/ts/api";
  import { app, browseTo, refreshShortcuts, setError, setStatus } from "$lib/ts/state.svelte";

  interface Props {
    collapsed: boolean;
  }

  let { collapsed }: Props = $props();

  function isActive(shortcut: api.Shortcut) {
    return (
      app.scope === "local" &&
      (app.currentDir === shortcut.browse_path || app.currentDir.startsWith(`${shortcut.browse_path}/`))
    );
  }

  function openShortcut(shortcut: api.Shortcut) {
    app.scope = "local";
    browseTo(shortcut.browse_path);
  }

  async function addShortcut() {
    const selected = await open({ directory: true, multiple: false, title: "Choose a folder to add" });
    if (!selected || Array.isArray(selected)) return;

    try {
      const shortcut = await api.addShortcut(selected);
      await refreshShortcuts();
      openShortcut(shortcut);
    } catch (error) {
      setError(error);
    }
  }

  async function removeShortcut(shortcut: api.Shortcut) {
    try {
      await api.removeShortcut(shortcut.id);
      if (isActive(shortcut)) await browseTo("");
      await refreshShortcuts();
      setStatus(`Removed the shortcut to ${shortcut.name}. The folder itself was not changed.`);
    } catch (error) {
      setError(error);
    }
  }
</script>

{#each app.shortcuts as shortcut (shortcut.id)}
  {@const active = isActive(shortcut)}
  <div
    class="group flex h-8 w-full items-center rounded-md text-[13px] transition-colors
      {active
      ? 'bg-[var(--color-surface-sunken)] font-medium text-[var(--color-ink)]'
      : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]/60 hover:text-[var(--color-ink)]'}"
  >
    <button
      class="flex h-full min-w-0 flex-1 items-center gap-2.5 {collapsed ? 'justify-center px-0' : 'pl-2.5'}"
      title={shortcut.available ? shortcut.path : `${shortcut.path} is not available`}
      aria-label={shortcut.name}
      onclick={() => openShortcut(shortcut)}
    >
      <Icon
        icon={shortcut.available ? "ph:folder-simple" : "ph:folder-simple-dashed"}
        class="shrink-0 text-base {active ? 'text-[var(--color-accent)]' : ''}"
      />
      {#if !collapsed}<span class="truncate {shortcut.available ? '' : 'opacity-60'}">{shortcut.name}</span>{/if}
    </button>
    {#if !collapsed}
      <button
        class="icon-btn icon-btn-sm mr-1 opacity-0 group-hover:opacity-100 focus-visible:opacity-100"
        title="Remove shortcut (the folder is kept)"
        aria-label="Remove shortcut to {shortcut.name}"
        onclick={() => removeShortcut(shortcut)}
      >
        <Icon icon="ph:x" class="text-xs" />
      </button>
    {/if}
  </div>
{/each}

<button
  onclick={addShortcut}
  title={collapsed ? "Add a folder" : undefined}
  aria-label="Add a folder"
  class="flex h-8 w-full items-center gap-2.5 rounded-md text-[13px] text-[var(--color-ink-subtle)] transition-colors hover:bg-[var(--color-surface-sunken)]/60 hover:text-[var(--color-ink)]
    {collapsed ? 'justify-center px-0' : 'px-2.5'}"
>
  <Icon icon="ph:plus" class="shrink-0 text-base" />
  {#if !collapsed}Add folder{/if}
</button>
