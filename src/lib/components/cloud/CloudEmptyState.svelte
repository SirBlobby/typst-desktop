<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { Snippet } from "svelte";

  let {
    icon,
    heading,
    message,
    children,
  }: { icon: string; heading: string; message: string; children?: Snippet } = $props();
</script>

<div class="flex min-h-[45vh] flex-col items-center justify-center rounded-xl border border-dashed border-[var(--color-line)] bg-[var(--color-surface)] px-6 text-center">
  <Icon {icon} class="mb-4 text-5xl text-[var(--color-accent)]" />
  <h3 class="mb-1 text-lg font-semibold text-[var(--color-ink)]">{heading}</h3>
  <p class="max-w-sm text-sm text-[var(--color-ink-muted)]">{message}</p>
  {#if children}
    <div class="mt-4 flex gap-2">
      {@render children()}
    </div>
  {/if}
</div>
