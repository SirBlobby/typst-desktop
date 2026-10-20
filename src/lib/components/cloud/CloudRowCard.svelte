<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { CloudEntry } from "$lib/ts/cloud-entries";
  import { folderIconStyle, folderTileStyle } from "$lib/ts/folder-colors";
  import CloudSyncBadge from "./CloudSyncBadge.svelte";

  interface Props {
    entry: CloudEntry;
    dropActive: boolean;
    canDrop: boolean;
    onmenu: (entry: CloudEntry, x: number, y: number) => void;
    ondragstart: (event: DragEvent, entry: CloudEntry) => void;
    ondragend: () => void;
    ondragover: (event: DragEvent, folderId: string) => void;
    ondragleave: (folderId: string) => void;
    ondrop: (event: DragEvent, folderId: string) => void;
  }

  let { entry, dropActive, canDrop, onmenu, ondragstart, ondragend, ondragover, ondragleave, ondrop }: Props = $props();

  const isFolder = $derived(entry.kind === "folder");

  function openMenuFromButton(event: MouseEvent) {
    event.stopPropagation();
    const bounds = (event.currentTarget as HTMLElement).getBoundingClientRect();
    onmenu(entry, bounds.right - 176, bounds.bottom + 4);
  }

  function openMenuAtCursor(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    onmenu(entry, event.clientX, event.clientY);
  }
</script>

<div
  class="group relative flex cursor-pointer flex-row items-center rounded-lg border bg-[var(--color-surface)] p-3 shadow-sm transition
    {entry.syncState === 'pending' ? 'border-[var(--color-accent)]' : 'border-[var(--color-line)]'}
    {dropActive ? 'bg-[var(--color-accent-soft)] ring-2 ring-[var(--color-accent)]' : 'hover:border-[var(--color-accent)]'}"
  role="button"
  tabindex="0"
  title={entry.name}
  draggable="true"
  onclick={entry.open}
  onkeydown={(event) => event.key === "Enter" && entry.open()}
  oncontextmenu={openMenuAtCursor}
  ondragstart={(event) => ondragstart(event, entry)}
  ondragend={ondragend}
  ondragover={(event) => canDrop && isFolder && ondragover(event, entry.id)}
  ondragleave={() => isFolder && ondragleave(entry.id)}
  ondrop={(event) => canDrop && isFolder && ondrop(event, entry.id)}
>
  {#if isFolder}
    <div
      class="pointer-events-none mr-3 flex h-10 w-10 shrink-0 items-center justify-center rounded-md transition-transform group-hover:scale-105"
      style={folderTileStyle(entry.color)}
    >
      <Icon icon="mdi:folder" class="text-2xl" style={folderIconStyle(entry.color)} />
    </div>
  {:else}
    <div class="pointer-events-none mr-3 flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-indigo-50 transition-transform group-hover:scale-105 dark:bg-indigo-500/10">
      <Icon icon="mdi:folder-star" class="text-2xl text-indigo-500" />
    </div>
  {/if}
  <div class="pointer-events-none min-w-0 flex-1 pr-8">
    <p class="truncate text-sm font-medium text-[var(--color-ink)]">{entry.name}</p>
    <div class="mt-0.5 flex items-center gap-1.5">
      <p class="truncate text-xs text-[var(--color-ink-muted)]">{entry.detail}</p>
      <CloudSyncBadge syncState={entry.syncState} />
    </div>
  </div>

  <button
    class="absolute right-2 top-1/2 -translate-y-1/2 rounded p-1 text-[var(--color-ink-muted)] opacity-0 transition-opacity hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] group-hover:opacity-100 group-focus-within:opacity-100"
    aria-label="{entry.kind === 'folder' ? 'Folder' : 'Project'} actions"
    onclick={openMenuFromButton}
  >
    <Icon icon="mdi:dots-vertical" class="text-lg" />
  </button>
</div>
