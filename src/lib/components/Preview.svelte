<script lang="ts">
  import Icon from "@iconify/svelte";
  import { untrack } from "svelte";
  import type { CompileResult, Diagnostic, PagePoint } from "#lib/ts/api";
  import { PreviewPageCache } from "#lib/ts/preview-pages";

  type JumpTarget = PagePoint & { seq: number };

  interface Props {
    compiled: CompileResult | null;
    diagnostics: Diagnostic[];
    compiling: boolean;
    jumpTo?: JumpTarget | null;
    onpageclick?: (point: PagePoint) => void;
  }

  let { compiled, diagnostics, compiling, jumpTo = null, onpageclick }: Props = $props();

  const MARKER_DURATION_MS = 900;
  const pageCache = new PreviewPageCache();

  let zoom = $state(1);
  let viewport: HTMLDivElement;
  let marker = $state<JumpTarget | null>(null);
  let markerTimer: ReturnType<typeof setTimeout> | undefined;

  const errors = $derived(diagnostics.filter((d) => d.severity === "error"));
  const pages = $derived(pageCache.prepare(compiled?.pages ?? []));

  $effect(() => {
    if (jumpTo) untrack(() => scrollToPoint(jumpTo!));
  });

  function scrollToPoint(target: JumpTarget) {
    const page = viewport?.querySelector<HTMLElement>(`[data-page="${target.page}"]`);
    if (!page) return;
    const pageRect = page.getBoundingClientRect();
    const viewRect = viewport.getBoundingClientRect();
    viewport.scrollTo({
      top: viewport.scrollTop + pageRect.top - viewRect.top + target.y * pageRect.height - viewport.clientHeight / 2,
      left: viewport.scrollLeft + pageRect.left - viewRect.left + target.x * pageRect.width - viewport.clientWidth / 2,
      behavior: "smooth",
    });
    marker = target;
    clearTimeout(markerTimer);
    markerTimer = setTimeout(() => (marker = null), MARKER_DURATION_MS);
  }

  function pageClick(event: MouseEvent, page: number) {
    if (!onpageclick || !window.getSelection()?.isCollapsed) return;
    if ((event.target as Element).closest("a")) return;
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    onpageclick({ page, x: (event.clientX - rect.left) / rect.width, y: (event.clientY - rect.top) / rect.height });
  }
</script>

<div class="flex h-full flex-col bg-[var(--color-canvas)]">
  <div
    class="bar gap-2 pl-3 pr-1.5 text-xs"
  >
    <Icon icon="ph:eye" class="text-sm text-[var(--color-ink-subtle)]" />
    <span class="text-[var(--color-ink-muted)]">
      {#if compiled}
        {compiled.stats.pages} {compiled.stats.pages === 1 ? "page" : "pages"} · {compiled.stats.words} {compiled.stats.words === 1 ? "word" : "words"}
      {:else}
        Preview
      {/if}
    </span>

    <div class="flex-1"></div>

    {#if compiling}
      <Icon icon="ph:circle-notch" class="animate-spin text-[var(--color-accent)]" />
    {/if}

    <button
      class="icon-btn icon-btn-sm"
      onclick={() => (zoom = Math.max(0.4, zoom - 0.15))}
      aria-label="Zoom out"
    >
      <Icon icon="ph:minus" />
    </button>
    <span class="w-10 text-center text-[11px] tabular-nums text-[var(--color-ink-muted)]">
      {Math.round(zoom * 100)}%
    </span>
    <button
      class="icon-btn icon-btn-sm"
      onclick={() => (zoom = Math.min(2.5, zoom + 0.15))}
      aria-label="Zoom in"
    >
      <Icon icon="ph:plus" />
    </button>
  </div>

  {#if errors.length > 0}
    <div
      class="scroll-thin max-h-40 shrink-0 overflow-y-auto border-b border-[var(--color-danger)]/20 bg-[var(--color-danger)]/6 px-3 py-2"
    >
      {#each errors as diagnostic}
        <div class="flex items-start gap-2 py-1 text-xs text-[var(--color-danger)]">
          <Icon icon="ph:warning-circle" class="mt-0.5 shrink-0" />
          <span>
            {#if diagnostic.line}
              <span class="font-medium">Line {diagnostic.line}:</span>
            {/if}
            {diagnostic.message}
          </span>
        </div>
      {/each}
    </div>
  {/if}

  <div bind:this={viewport} class="scroll-thin flex-1 overflow-auto px-8 py-6">
    {#if pages.length > 0}
      <div
        class="mx-auto flex flex-col items-center gap-6"
        style="width: {Math.round(zoom * 100)}%; max-width: {zoom > 1 ? 'none' : '820px'};"
      >
        {#each pages as page, index}
          <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
          <div
            data-page={index + 1}
            onclick={(event) => pageClick(event, index + 1)}
            class="preview-page relative w-full overflow-hidden rounded-sm bg-white shadow-lg ring-1 ring-black/5"
            class:sized={page.size !== null}
            style={page.size ? `aspect-ratio: ${page.size.width} / ${page.size.height};` : ""}
          >
            {@html page.svg}
            {#if marker?.page === index + 1}
              {#key marker.seq}
                <span class="jump-marker" style="left: {marker.x * 100}%; top: {marker.y * 100}%;"></span>
              {/key}
            {/if}
          </div>
        {/each}
      </div>
    {:else}
      <div
        class="empty-state h-full"
      >
        <Icon icon="ph:file-dashed" class="text-3xl" />
        <p class="text-sm">
          {errors.length > 0 ? "Fix the errors above to see a preview" : "Nothing to preview yet"}
        </p>
      </div>
    {/if}
  </div>
</div>

<style>
  .preview-page.sized {
    content-visibility: auto;
  }

  .jump-marker {
    position: absolute;
    width: 1.5rem;
    height: 1.5rem;
    margin: -0.75rem 0 0 -0.75rem;
    border-radius: 9999px;
    background: var(--color-accent);
    pointer-events: none;
    animation: jump-marker 900ms ease-out forwards;
  }

  @keyframes jump-marker {
    from { opacity: 0.6; transform: scale(0.5); }
    to { opacity: 0; transform: scale(2); }
  }
</style>
