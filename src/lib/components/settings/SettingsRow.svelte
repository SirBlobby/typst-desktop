<script lang="ts">
  import type { Snippet } from "svelte";

  let { label, description, stacked = false, children }: {
    label: string;
    description?: string;
    stacked?: boolean;
    children: Snippet;
  } = $props();
</script>

<div class="flex gap-3 px-4 py-3 {stacked ? 'flex-col' : 'items-center justify-between'}">
  <div class="min-w-0 {stacked ? '' : 'max-w-[55%]'}">
    <p class="text-xs font-medium">{label}</p>
    {#if description}
      <p class="mt-0.5 text-[11px] text-[var(--color-ink-muted)]">{description}</p>
    {/if}
  </div>
  <div class="flex min-w-0 items-center gap-2 {stacked ? '' : 'justify-end'}">
    {@render children()}
  </div>
</div>
