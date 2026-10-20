<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    icon?: string;
    width?: string;
    onclose: () => void;
    children: Snippet;
    footer?: Snippet;
  }

  let {
    title,
    icon = "ph:squares-four",
    width = "max-w-md",
    onclose,
    children,
    footer,
  }: Props = $props();

  function handleKey(event: KeyboardEvent) {
    if (event.key === "Escape") onclose();
  }
</script>

<svelte:window on:keydown={handleKey} />

<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-[var(--backdrop)] p-6 backdrop-blur-[2px]"
  role="presentation"
  onclick={(event) => {
    if (event.target === event.currentTarget) onclose();
  }}
>
  <div
    class="w-full {width} overflow-hidden rounded-2xl border border-[var(--color-line)] bg-[var(--color-surface-raised)] shadow-2xl"
    role="dialog"
    aria-modal="true"
    aria-label={title}
  >
    <header
      class="flex items-center gap-2.5 px-5 pb-1 pt-4"
    >
      <span class="flex h-7 w-7 items-center justify-center rounded-lg bg-[var(--color-accent-soft)]">
        <Icon {icon} class="text-base text-[var(--color-accent)]" />
      </span>
      <h2 class="flex-1 text-sm font-semibold">{title}</h2>
      <button
        class="icon-btn -mr-1.5"
        onclick={onclose}
        aria-label="Close"
      >
        <Icon icon="ph:x" class="text-base" />
      </button>
    </header>

    <div class="scroll-thin max-h-[70vh] overflow-y-auto px-5 pb-5 pt-3">
      {@render children()}
    </div>

    {#if footer}
      <footer
        class="flex items-center justify-end gap-2 border-t border-[var(--color-line)] bg-[var(--color-surface)] px-5 py-3"
      >
        {@render footer()}
      </footer>
    {/if}
  </div>
</div>
