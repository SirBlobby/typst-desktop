<script lang="ts">
  import Icon from "@iconify/svelte";
  import { fileIcon, formatShortDate, isFontName, type CloudEntry } from "$lib/ts/cloud-entries";
  import CloudSyncBadge from "./CloudSyncBadge.svelte";

  interface Props {
    entry: CloudEntry;
    thumbnail: { kind: string; data: string } | null;
    onmenu: (entry: CloudEntry, x: number, y: number) => void;
    ondragstart: (event: DragEvent, entry: CloudEntry) => void;
    ondragend: () => void;
  }

  let { entry, thumbnail, onmenu, ondragstart, ondragend }: Props = $props();

  const isDocument = $derived(entry.kind === "document");
  const placeholderIcon = $derived(isDocument ? "mdi:file-document" : fileIcon(entry.name));
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
  class="group relative flex cursor-pointer flex-col rounded-lg border bg-[var(--color-surface)] shadow-sm transition hover:border-[var(--color-accent)]
    {entry.syncState === 'pending' ? 'border-[var(--color-accent)]' : 'border-[var(--color-line)]'}"
  role="button"
  tabindex="0"
  draggable="true"
  onclick={entry.open}
  onkeydown={(event) => event.key === "Enter" && entry.open()}
  oncontextmenu={openMenuAtCursor}
  ondragstart={(event) => ondragstart(event, entry)}
  ondragend={ondragend}
>
  <div class="pointer-events-none relative flex h-40 w-full items-center justify-center overflow-hidden rounded-t-lg border-b border-[var(--color-line)] bg-[var(--color-surface-muted)]">
    {#if thumbnail?.kind === "svg"}
      <div class="flex h-full w-full items-start justify-center bg-white transition-transform duration-300 group-hover:scale-110 [&_svg]:h-auto [&_svg]:w-full">
        {@html thumbnail.data}
      </div>
    {:else if thumbnail}
      <img src={thumbnail.data} alt={entry.name} class="h-full w-full object-contain transition-transform duration-300 group-hover:scale-110" />
    {:else}
      <div
        class="rounded-lg p-4 transition-transform duration-300 group-hover:scale-110
          {isDocument
          ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
          : 'bg-green-50 text-green-600 dark:bg-green-500/10 dark:text-green-400'}"
      >
        <Icon icon={placeholderIcon} class="text-4xl" />
      </div>
    {/if}
    {#if entry.syncState}
      <span class="absolute left-2 top-2">
        <CloudSyncBadge syncState={entry.syncState} />
      </span>
    {/if}
  </div>

  <div class="flex flex-grow flex-col p-4">
    <div class="flex items-start justify-between gap-2">
      <h3 class="pointer-events-none truncate text-lg font-semibold text-[var(--color-ink)]" title={entry.name}>{entry.name}</h3>
      <button
        class="shrink-0 rounded-md p-1 text-[var(--color-ink-muted)] transition-colors hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]"
        aria-label="{isDocument ? 'Document' : 'File'} actions"
        onclick={openMenuFromButton}
      >
        <Icon icon="mdi:dots-vertical" class="text-xl" />
      </button>
    </div>
    <p class="pointer-events-none mt-2 flex items-center gap-1 text-xs text-[var(--color-ink-muted)]">
      <Icon icon={isDocument ? "mdi:clock-outline" : isFontName(entry.name) ? "mdi:format-font" : "mdi:image-outline"} class="text-sm" />
      {dateLabel}
      {#if entry.detail !== "Document" && isDocument}
        <span class="opacity-60">·</span>
        {entry.detail}
      {/if}
    </p>
  </div>
</div>
