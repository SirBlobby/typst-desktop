<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { CloudEntry } from "#lib/ts/cloud-entries";
  import { folderIconStyle, folderTileStyle } from "#lib/ts/folder-colors";
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
  class="card group relative flex cursor-pointer flex-row items-center py-2.5 pl-2.5 pr-3
    {entry.syncState === 'pending' ? 'border-[var(--color-accent)]/50' : ''}
    {dropActive ? 'border-[var(--color-accent)] bg-[var(--color-accent-soft)]' : 'card-interactive'}"
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
      class="pointer-events-none mr-2.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-md"
      style={folderTileStyle(entry.color)}
    >
      <Icon icon="ph:folder-fill" class="text-xl" style={folderIconStyle(entry.color)} />
    </div>
  {:else}
    <div class="pointer-events-none mr-2.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-[var(--color-accent-soft)]">
      <Icon icon="ph:folder-star-fill" class="text-xl text-[var(--color-accent)]" />
    </div>
  {/if}
  <div class="pointer-events-none min-w-0 flex-1 pr-8">
    <p class="truncate text-[13px] font-medium text-[var(--color-ink)]">{entry.name}</p>
    <div class="mt-0.5 flex items-center gap-1.5">
      <p class="truncate text-[11px] text-[var(--color-ink-muted)]">{entry.detail}</p>
      <CloudSyncBadge syncState={entry.syncState} />
    </div>
  </div>

  <button
    class="icon-btn icon-btn-sm absolute right-2 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 group-focus-within:opacity-100"
    aria-label="{entry.kind === 'folder' ? 'Folder' : 'Project'} actions"
    onclick={openMenuFromButton}
  >
    <Icon icon="ph:dots-three" class="text-base" />
  </button>
</div>
