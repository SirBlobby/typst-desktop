<script lang="ts">
  import Icon from "@iconify/svelte";
  import { fileIcon, formatShortDate, type CloudEntry } from "$lib/ts/cloud-entries";
  import { folderIconStyle } from "$lib/ts/folder-colors";
  import CloudSyncBadge from "./CloudSyncBadge.svelte";

  interface Props {
    entries: CloudEntry[];
    dropTarget: string | null;
    canDrop: boolean;
    onmenu: (entry: CloudEntry, x: number, y: number) => void;
    ondragstart: (event: DragEvent, entry: CloudEntry) => void;
    ondragend: () => void;
    ondragover: (event: DragEvent, folderId: string) => void;
    ondragleave: (folderId: string) => void;
    ondrop: (event: DragEvent, folderId: string) => void;
  }

  let { entries, dropTarget, canDrop, onmenu, ondragstart, ondragend, ondragover, ondragleave, ondrop }: Props = $props();

  const kindIcons = {
    folder: { icon: "mdi:folder", color: "text-yellow-500" },
    project: { icon: "mdi:folder-star", color: "text-indigo-500" },
    document: { icon: "mdi:file-document", color: "text-[var(--color-accent)]" },
    file: { icon: "", color: "text-green-500" },
  };

  const columns = "grid-cols-[minmax(0,1fr)_7rem_9rem_6rem_2.5rem]";

  function openMenuFromButton(event: MouseEvent, entry: CloudEntry) {
    event.stopPropagation();
    const bounds = (event.currentTarget as HTMLElement).getBoundingClientRect();
    onmenu(entry, bounds.right - 208, bounds.bottom + 4);
  }

  function openMenuAtCursor(event: MouseEvent, entry: CloudEntry) {
    event.preventDefault();
    event.stopPropagation();
    onmenu(entry, event.clientX, event.clientY);
  }
</script>

<div class="overflow-hidden rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)]" role="table" aria-label="Drive items">
  <div role="row" class="grid {columns} gap-3 border-b border-[var(--color-line)] bg-[var(--color-surface-muted)] px-4 py-2 text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">
    <span role="columnheader">Name</span>
    <span role="columnheader">Type</span>
    <span role="columnheader">Modified</span>
    <span role="columnheader">Status</span>
    <span role="columnheader" class="sr-only">Actions</span>
  </div>

  {#each entries as entry (entry.key)}
    {@const isFolder = entry.kind === "folder"}
    {@const style = kindIcons[entry.kind]}
    <div
      role="row"
      tabindex="0"
      draggable="true"
      class="group grid {columns} cursor-pointer items-center gap-3 border-b border-[var(--color-line)] px-4 py-2 text-sm transition-colors last:border-b-0
        {dropTarget === entry.id && isFolder ? 'bg-[var(--color-accent-soft)]' : 'hover:bg-[var(--color-surface-muted)]'}"
      onclick={entry.open}
      onkeydown={(event) => event.key === "Enter" && entry.open()}
      oncontextmenu={(event) => openMenuAtCursor(event, entry)}
      ondragstart={(event) => ondragstart(event, entry)}
      ondragend={ondragend}
      ondragover={(event) => canDrop && isFolder && ondragover(event, entry.id)}
      ondragleave={() => isFolder && ondragleave(entry.id)}
      ondrop={(event) => canDrop && isFolder && ondrop(event, entry.id)}
    >
      <span role="cell" class="flex min-w-0 items-center gap-3">
        {#if isFolder}
          <Icon icon={style.icon} class="shrink-0 text-xl" style={folderIconStyle(entry.color)} />
        {:else}
          <Icon icon={entry.kind === "file" ? fileIcon(entry.name) : style.icon} class="shrink-0 text-xl {style.color}" />
        {/if}
        <span class="truncate font-medium text-[var(--color-ink)]" title={entry.name}>{entry.name}</span>
      </span>
      <span role="cell" class="truncate text-[var(--color-ink-muted)]">{entry.detail}</span>
      <span role="cell" class="truncate text-[var(--color-ink-muted)]">{formatShortDate(entry.modified)}</span>
      <span role="cell" class="flex min-w-0"><CloudSyncBadge syncState={entry.syncState} /></span>
      <span role="cell" class="flex justify-end">
        <button
          class="rounded p-1 text-[var(--color-ink-muted)] opacity-0 transition-opacity hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] group-hover:opacity-100 group-focus-within:opacity-100"
          aria-label="Actions for {entry.name}"
          onclick={(event) => openMenuFromButton(event, entry)}
        >
          <Icon icon="mdi:dots-vertical" class="text-base" />
        </button>
      </span>
    </div>
  {/each}
</div>
