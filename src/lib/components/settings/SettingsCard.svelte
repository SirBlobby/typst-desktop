<script lang="ts">
  import type { Snippet } from "svelte";

  let { title, children }: { title?: string; children: Snippet } = $props();
</script>

<section>
  {#if title}
    <h3 class="section-label mb-1.5 px-1">{title}</h3>
  {/if}
  <div class="card divide-y divide-[var(--color-line)] overflow-hidden shadow-sm">
    {@render children()}
  </div>
</section>
