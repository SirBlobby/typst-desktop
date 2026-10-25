<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { Snippet } from "svelte";
  import * as api from "#lib/ts/api";
  import { openPdf, pageSizes, renderPdfPage, type PageSize, type PdfDocument } from "#lib/ts/pdf";
  import ZoomControls from "./ZoomControls.svelte";

  interface Props {
    path: string;
    actions?: Snippet;
  }

  let { path, actions }: Props = $props();

  const PAGE_PADDING = 48;

  let pdf = $state.raw<PdfDocument | null>(null);
  let sizes = $state<PageSize[]>([]);
  let error = $state("");
  let loading = $state(true);
  let zoom = $state(1);
  let fit = $state(true);
  let viewportWidth = $state(0);

  $effect(() => {
    const requested = path;
    let opened: PdfDocument | null = null;
    let cancelled = false;

    loading = true;
    error = "";
    pdf = null;
    sizes = [];

    (async () => {
      try {
        opened = await openPdf(requested);
        const measured = await pageSizes(opened);
        if (cancelled) return;
        pdf = opened;
        sizes = measured;
        zoom = 1;
        fit = true;
      } catch (caught) {
        if (!cancelled) error = api.errorMessage(caught);
      } finally {
        if (!cancelled) loading = false;
      }
    })();

    return () => {
      cancelled = true;
      opened?.destroy();
    };
  });

  const widestPage = $derived(Math.max(1, ...sizes.map((size) => size.width)));
  const fitScale = $derived(
    viewportWidth > PAGE_PADDING ? Math.min(1.5, (viewportWidth - PAGE_PADDING) / widestPage) : 1,
  );
  const scale = $derived(fit ? fitScale : zoom);

  function changeZoom(delta: number) {
    const start = fit ? fitScale : zoom;
    fit = false;
    zoom = Math.min(4, Math.max(0.25, Math.round((start + delta) * 4) / 4));
  }
</script>

<div class="flex h-full min-h-0 flex-col bg-[var(--color-canvas)]">
  <div class="bar gap-2 pl-3 pr-1.5 text-xs">
    <Icon icon="ph:file-pdf" class="shrink-0 text-sm text-[var(--color-accent)]" />
    <span class="truncate font-medium">{path.split("/").pop()}</span>
    {#if sizes.length > 0}
      <span class="shrink-0 text-[var(--color-ink-muted)]">
        {sizes.length} {sizes.length === 1 ? "page" : "pages"}
      </span>
    {/if}
    <div class="flex-1"></div>
    <ZoomControls
      {fit}
      zoom={scale}
      onzoom={changeZoom}
      ontogglefit={() => {
        zoom = 1;
        fit = !fit;
      }}
    />
    {@render actions?.()}
  </div>

  <div class="scroll-thin min-h-0 flex-1 overflow-auto" bind:clientWidth={viewportWidth}>
    {#if loading}
      <div class="flex h-full items-center justify-center">
        <Icon icon="ph:circle-notch" class="animate-spin text-2xl text-[var(--color-ink-subtle)]" />
      </div>
    {:else if error}
      <div class="empty-state h-full">
        <Icon icon="ph:warning-circle" class="text-3xl" />
        <p class="text-sm">{error}</p>
      </div>
    {:else if pdf}
      <div class="flex w-max min-w-full flex-col items-center gap-6 px-6 py-6">
        {#each sizes as size, index (index)}
          <canvas
            class="block rounded-sm bg-white shadow-lg ring-1 ring-black/5"
            style="width: {size.width * scale}px; height: {size.height * scale}px;"
            aria-label="Page {index + 1}"
            use:renderPdfPage={{ document: pdf, pageNumber: index + 1, scale }}
          ></canvas>
        {/each}
      </div>
    {/if}
  </div>
</div>
