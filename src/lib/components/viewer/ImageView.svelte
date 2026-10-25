<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { Snippet } from "svelte";
  import * as api from "#lib/ts/api";
  import type { ImageData } from "#lib/ts/api";
  import { formatFileSize } from "#lib/ts/viewer";
  import ZoomControls from "./ZoomControls.svelte";

  interface Props {
    path: string;
    actions?: Snippet;
  }

  let { path, actions }: Props = $props();

  let image = $state<ImageData | null>(null);
  let error = $state("");
  let loading = $state(true);
  let zoom = $state(1);
  let fit = $state(true);

  $effect(() => {
    const requested = path;
    loading = true;
    error = "";
    api
      .readImage(requested)
      .then((result) => {
        if (requested !== path) return;
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

  function changeZoom(delta: number) {
    fit = false;
    zoom = Math.min(8, Math.max(0.1, zoom + delta));
  }
</script>

<div class="flex h-full min-h-0 flex-col bg-[var(--color-canvas)]">
  <div class="bar gap-2 pl-3 pr-1.5 text-xs">
    <Icon icon="ph:image" class="shrink-0 text-sm text-[var(--color-accent)]" />
    <span class="truncate font-medium">{image?.name ?? path.split("/").pop()}</span>
    {#if image}
      <span class="shrink-0 text-[var(--color-ink-muted)]">
        {#if image.width && image.height}{image.width} × {image.height} ·{/if}
        {formatFileSize(image.size)}
      </span>
    {/if}
    <div class="flex-1"></div>
    <ZoomControls
      {fit}
      {zoom}
      onzoom={changeZoom}
      ontogglefit={() => {
        fit = !fit;
        zoom = 1;
      }}
    />
    {@render actions?.()}
  </div>

  <div class="relative flex min-h-0 flex-1 items-center justify-center">
    {#if loading}
      <Icon icon="ph:circle-notch" class="animate-spin text-2xl text-[var(--color-ink-subtle)]" />
    {:else if error}
      <div class="empty-state">
        <Icon icon="ph:warning-circle" class="text-3xl" />
        <p class="text-sm">{error}</p>
      </div>
    {:else if image}
      <div class="scroll-thin flex h-full w-full overflow-auto p-6">
        <img
          src={image.data}
          alt={image.name}
          class={fit ? "m-auto max-h-full max-w-full object-contain" : "m-auto max-w-none"}
          style={fit ? "" : `width: ${zoom * 100}%`}
        />
      </div>
    {/if}
  </div>
</div>
