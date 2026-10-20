<script lang="ts">
  let { label, checked, onchange }: { label: string; checked: boolean; onchange: (value: boolean) => void } = $props();
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={label}
  onclick={() => onchange(!checked)}
  class="relative inline-flex h-5 w-9 shrink-0 items-center rounded-full transition-colors {checked
    ? 'bg-[var(--color-accent)]'
    : 'bg-[var(--color-line-strong)]'}"
>
  <span class="inline-block h-4 w-4 rounded-full bg-white shadow-sm transition-transform {checked ? 'translate-x-4' : 'translate-x-0.5'}"></span>
</button>
