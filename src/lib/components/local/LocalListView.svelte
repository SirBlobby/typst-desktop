<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { Snippet } from "svelte";
  import type { BrowseEntry } from "$lib/ts/api";
  import { folderIconStyle } from "$lib/ts/folder-colors";
  import { entryIcon, entryTypeLabel, formatDate, isContainer, sizeLabel } from "$lib/ts/local-entries";
  import LocalSyncBadge from "./LocalSyncBadge.svelte";

  interface Props {
    entries: BrowseEntry[];
    dropTarget: string | null;
    actions: Snippet<[BrowseEntry]>;
    onactivate: (entry: BrowseEntry) => void;
    oncontextmenu: (event: MouseEvent, path: string) => void;
    ondragstart: (event: DragEvent, path: string) => void;
    ondragend: () => void;
    ondragover: (event: DragEvent, path: string) => void;
    ondragleave: (path: string) => void;
    ondrop: (event: DragEvent, path: string) => void;
  }

  let {
    entries,
    dropTarget,
    actions,
    onactivate,
    oncontextmenu,
    ondragstart,
    ondragend,
    ondragover,
    ondragleave,
    ondrop,
  }: Props = $props();

  const columns = "grid-cols-[minmax(0,1fr)_7rem_8rem_6rem_2.5rem]";

  function iconClass(entry: BrowseEntry) {
    if (entry.kind === "project" || entry.kind === "document") return "text-[var(--color-accent)]";
    if (entry.kind === "file") return "text-[var(--color-ink-muted)]";
    return "";
  }
</script>

<div class="card overflow-hidden" role="table" aria-label="Workspace items">
  <div role="row" class="grid {columns} section-label gap-3 border-b border-[var(--color-line)] px-4 py-2">
    <span role="columnheader">Name</span>
    <span role="columnheader">Type</span>
    <span role="columnheader">Modified</span>
    <span role="columnheader">Size</span>
    <span role="columnheader" class="sr-only">Actions</span>
  </div>

  {#each entries as entry (entry.path)}
    {@const container = isContainer(entry)}
    <div
      role="row"
      tabindex="0"
      draggable="true"
      class="group grid {columns} cursor-pointer items-center gap-3 border-b border-[var(--color-line)] px-4 py-2 text-[13px] transition-colors last:border-b-0
        {dropTarget === entry.path ? 'bg-[var(--color-accent-soft)]' : 'hover:bg-[var(--color-surface-sunken)]/60'}"
      onclick={() => onactivate(entry)}
      onkeydown={(event) => event.key === "Enter" && onactivate(entry)}
      oncontextmenu={(event) => oncontextmenu(event, entry.path)}
      ondragstart={(event) => ondragstart(event, entry.path)}
      {ondragend}
      ondragover={(event) => container && ondragover(event, entry.path)}
      ondragleave={() => container && ondragleave(entry.path)}
      ondrop={(event) => container && ondrop(event, entry.path)}
    >
      <span role="cell" class="flex min-w-0 items-center gap-3">
        {#if entry.kind === "folder"}
          <Icon icon="ph:folder-fill" class="shrink-0 text-lg" style={folderIconStyle(entry.color)} />
        {:else if entry.kind === "project"}
          <Icon icon="ph:folder-star-fill" class="shrink-0 text-lg text-[var(--color-accent)]" />
        {:else}
          <Icon icon={entryIcon(entry)} class="shrink-0 text-lg {iconClass(entry)}" />
        {/if}
        <span class="truncate font-medium text-[var(--color-ink)]" title={entry.name}>{entry.name}</span>
        <LocalSyncBadge {entry} />
      </span>
      <span role="cell" class="truncate text-[var(--color-ink-muted)]">{entryTypeLabel(entry)}</span>
      <span role="cell" class="truncate text-[var(--color-ink-muted)]">{formatDate(entry.modified)}</span>
      <span role="cell" class="truncate tabular-nums text-[var(--color-ink-muted)]">{sizeLabel(entry)}</span>
      <span role="cell" class="flex justify-end">
        {@render actions(entry)}
      </span>
    </div>
  {/each}
</div>
