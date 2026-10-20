<script lang="ts">
  import Icon from "@iconify/svelte";
  import { untrack } from "svelte";
  import * as api from "$lib/ts/api";
  import ImageView from "./ImageView.svelte";
  import PdfView from "./PdfView.svelte";

  interface Props {
    paths: string[];
    index: number;
    onclose: () => void;
  }

  let { paths, index, onclose }: Props = $props();

  let current = $state(untrack(() => index));
  const path = $derived(paths[current] ?? "");

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
</script>

<svelte:window onkeydown={handleKey} />

{#snippet actions()}
  {#if paths.length > 1}
    <div class="divider-v mx-1"></div>
    <button class="icon-btn icon-btn-sm" disabled={current === 0} onclick={() => step(-1)} aria-label="Previous file">
      <Icon icon="ph:caret-left" />
    </button>
    <span class="text-[11px] tabular-nums text-[var(--color-ink-muted)]">{current + 1} of {paths.length}</span>
    <button
      class="icon-btn icon-btn-sm"
      disabled={current === paths.length - 1}
      onclick={() => step(1)}
      aria-label="Next file"
    >
      <Icon icon="ph:caret-right" />
    </button>
  {/if}
  <div class="divider-v mx-1"></div>
  <button class="icon-btn icon-btn-sm" onclick={onclose} aria-label="Close">
    <Icon icon="ph:x" />
  </button>
{/snippet}

<div
  class="fixed inset-0 z-50 flex bg-[var(--backdrop)] p-6 backdrop-blur-[2px]"
  role="presentation"
  onclick={(event) => {
    if (event.target === event.currentTarget) onclose();
  }}
>
  <div class="card flex min-h-0 flex-1 flex-col overflow-hidden shadow-2xl">
    {#key path}
      {#if api.isPdfPath(path)}
        <PdfView {path} {actions} />
      {:else}
        <ImageView {path} {actions} />
      {/if}
    {/key}
  </div>
</div>
