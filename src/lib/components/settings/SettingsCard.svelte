<script lang="ts">
  import type { Snippet } from "svelte";

  let { title, children }: { title?: string; children: Snippet } = $props();
</script>

<section class="overflow-hidden rounded-md border border-[var(--color-line)] bg-[var(--color-surface)]">
  {#if title}
    <h3 class="border-b border-[var(--color-line)] bg-[var(--color-surface-muted)] px-4 py-2 text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">
      {title}
    </h3>
  {/if}
  <div class="divide-y divide-[var(--color-line)]">
    {@render children()}
  </div>
</section>
