<script lang="ts">
  import Icon from "@iconify/svelte";
  import { untrack } from "svelte";
  import * as api from "$lib/ts/api";
  import type { ImageData } from "$lib/ts/api";

  interface Props {
    paths: string[];
    index: number;
    onclose: () => void;
  }

  let { paths, index, onclose }: Props = $props();

  let current = $state(untrack(() => index));
  let image = $state<ImageData | null>(null);
  let error = $state("");
  let loading = $state(true);
  let zoom = $state(1);
  let fit = $state(true);

  $effect(() => {
    const path = paths[current];
    if (!path) return;

    loading = true;
    error = "";
    api
      .readImage(path)
      .then((result) => {
        image = result;
        zoom = 1;
        fit = true;
      })
      .catch((caught) => {
        image = null;
        error = api.errorMessage(caught);
      })
      .finally(() => (loading = false));
  });

  function step(delta: number) {
    const next = current + delta;
    if (next < 0 || next >= paths.length) return;
    current = next;
  }

  function handleKey(event: KeyboardEvent) {
    if (event.key === "Escape") onclose();
    if (event.key === "ArrowRight") step(1);
    if (event.key === "ArrowLeft") step(-1);
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  }
</script>

<svelte:window on:keydown={handleKey} />

<div
  class="fixed inset-0 z-50 flex flex-col bg-black/80"
  role="presentation"
  onclick={(event) => {
    if (event.target === event.currentTarget) onclose();
  }}
>
  <header
    class="flex h-11 shrink-0 items-center gap-3 bg-[var(--color-surface)] px-3 text-xs"
  >
    <Icon icon="ph:image" class="text-base text-[var(--color-accent)]" />
    <span class="font-medium">{image?.name ?? paths[current]?.split("/").pop()}</span>

    {#if image}
      <span class="text-[var(--color-ink-muted)]">
        {#if image.width && image.height}
          {image.width} × {image.height} ·
        {/if}
        {formatSize(image.size)}
      </span>
    {/if}

    <div class="flex-1"></div>

    {#if paths.length > 1}
      <span class="text-[var(--color-ink-muted)]">
        {current + 1} of {paths.length}
      </span>
    {/if}

    <button
      class="rounded p-1.5 text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]"
      onclick={() => {
        fit = false;
        zoom = Math.max(0.1, zoom - 0.25);
      }}
      aria-label="Zoom out"
    >
      <Icon icon="ph:minus" />
    </button>
    <button
      class="rounded px-2 py-1 text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]"
      onclick={() => {
        fit = !fit;
        zoom = 1;
      }}
    >
      {fit ? "Fit" : `${Math.round(zoom * 100)}%`}
    </button>
    <button
      class="rounded p-1.5 text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]"
      onclick={() => {
        fit = false;
        zoom = Math.min(8, zoom + 0.25);
      }}
      aria-label="Zoom in"
    >
      <Icon icon="ph:plus" />
    </button>

    <button
      class="rounded p-1.5 text-[var(--color-ink-muted)] transition hover:bg-[var(--color-danger)] hover:text-white"
      onclick={onclose}
      aria-label="Close"
    >
      <Icon icon="ph:x" />
    </button>
  </header>

  <div class="relative flex min-h-0 flex-1 items-center justify-center">
    {#if paths.length > 1}
      <button
        class="absolute left-3 z-10 rounded-full bg-black/50 p-2 text-white transition hover:bg-black/70 disabled:opacity-30"
        disabled={current === 0}
        onclick={() => step(-1)}
        aria-label="Previous image"
      >
        <Icon icon="ph:caret-left" class="text-lg" />
      </button>
      <button
        class="absolute right-3 z-10 rounded-full bg-black/50 p-2 text-white transition hover:bg-black/70 disabled:opacity-30"
        disabled={current === paths.length - 1}
        onclick={() => step(1)}
        aria-label="Next image"
      >
        <Icon icon="ph:caret-right" class="text-lg" />
      </button>
    {/if}

    {#if loading}
      <Icon icon="ph:circle-notch" class="animate-spin text-3xl text-white/70" />
    {:else if error}
      <div class="flex flex-col items-center gap-2 text-white/70">
        <Icon icon="ph:warning-circle" class="text-3xl" />
        <p class="text-sm">{error}</p>
      </div>
    {:else if image}
      <div class="scroll-thin h-full w-full overflow-auto p-6">
        <img
          src={image.data}
          alt={image.name}
          class={fit
            ? "mx-auto max-h-full max-w-full object-contain"
            : "mx-auto max-w-none"}
          style={fit ? "" : `width: ${zoom * 100}%`}
        />
      </div>
    {/if}
  </div>
</div>
