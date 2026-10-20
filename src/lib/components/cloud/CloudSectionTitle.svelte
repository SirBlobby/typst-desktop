<script lang="ts">
  let { title }: { title: string } = $props();
</script>

<div class="mb-3 flex items-center gap-2">
  <h2 class="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">{title}</h2>
  <div class="h-px flex-1 bg-[var(--color-line)]"></div>
</div>
