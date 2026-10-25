<script lang="ts">
  import Icon from "@iconify/svelte";
  import { fileIcon, formatShortDate, isFontName, type CloudEntry } from "#lib/ts/cloud-entries";
  import CloudSyncBadge from "./CloudSyncBadge.svelte";
  import PageThumbnail from "../PageThumbnail.svelte";

  interface Props {
    entry: CloudEntry;
    thumbnail: { kind: string; data: string } | null;
    onmenu: (entry: CloudEntry, x: number, y: number) => void;
    ondragstart: (event: DragEvent, entry: CloudEntry) => void;
    ondragend: () => void;
  }

  let { entry, thumbnail, onmenu, ondragstart, ondragend }: Props = $props();

  const isDocument = $derived(entry.kind === "document");
  const placeholderIcon = $derived(isDocument ? "ph:file-text" : fileIcon(entry.name));
  const dateLabel = $derived(
    `${isDocument ? "Edited" : "Uploaded"} ${formatShortDate(entry.modified)}`,
  );

  function openMenuFromButton(event: MouseEvent) {
    event.stopPropagation();
    const bounds = (event.currentTarget as HTMLElement).getBoundingClientRect();
    onmenu(entry, bounds.right - 208, bounds.bottom + 4);
  }

  function openMenuAtCursor(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    onmenu(entry, event.clientX, event.clientY);
  }
</script>

<div
  class="card card-interactive group relative flex cursor-pointer flex-col overflow-hidden
    {entry.syncState === 'pending' ? 'border-[var(--color-accent)]/50' : ''}"
  role="button"
  tabindex="0"
  draggable="true"
  onclick={entry.open}
  onkeydown={(event) => event.key === "Enter" && entry.open()}
  oncontextmenu={openMenuAtCursor}
  ondragstart={(event) => ondragstart(event, entry)}
  ondragend={ondragend}
>
  <div class="relative">
    <PageThumbnail
      {thumbnail}
      name={entry.name}
      fallbackIcon={placeholderIcon}
      fallbackClass={isDocument
        ? "bg-[var(--color-accent-soft)] text-[var(--color-accent)]"
        : "bg-[var(--color-success)]/12 text-[var(--color-success)]"}
    />
    {#if entry.syncState}
      <span class="absolute left-2 top-2">
        <CloudSyncBadge syncState={entry.syncState} />
      </span>
    {/if}
  </div>

  <div class="flex flex-grow flex-col py-2.5 pl-3 pr-2">
    <div class="flex items-center justify-between gap-1">
      <h3 class="pointer-events-none truncate text-[13px] font-medium text-[var(--color-ink)]" title={entry.name}>{entry.name}</h3>
      <button
        class="icon-btn icon-btn-sm opacity-0 group-hover:opacity-100 group-focus-within:opacity-100"
        aria-label="{isDocument ? 'Document' : 'File'} actions"
        onclick={openMenuFromButton}
      >
        <Icon icon="ph:dots-three" class="text-base" />
      </button>
    </div>
    <p class="pointer-events-none mt-0.5 flex items-center gap-1 text-[11px] text-[var(--color-ink-muted)]">
      <Icon icon={isDocument ? "ph:clock" : isFontName(entry.name) ? "ph:text-aa" : "ph:image"} class="text-xs" />
      {dateLabel}
      {#if entry.detail !== "Document" && isDocument}
        <span class="opacity-60">·</span>
        {entry.detail}
      {/if}
    </p>
  </div>
</div>
