<script lang="ts">
  import Icon from "@iconify/svelte";

  interface Props {
    fit: boolean;
    zoom: number;
    onzoom: (delta: number) => void;
    ontogglefit: () => void;
  }

  let { fit, zoom, onzoom, ontogglefit }: Props = $props();
</script>

<button class="icon-btn icon-btn-sm" onclick={() => onzoom(-0.25)} aria-label="Zoom out">
  <Icon icon="ph:minus" />
</button>
<button
  class="w-12 rounded px-1 py-0.5 text-center text-[11px] tabular-nums text-[var(--color-ink-muted)] transition-colors hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]"
  title={fit ? "Show at actual size" : "Fit to window"}
  onclick={ontogglefit}
>
  {fit ? "Fit" : `${Math.round(zoom * 100)}%`}
</button>
<button class="icon-btn icon-btn-sm" onclick={() => onzoom(0.25)} aria-label="Zoom in">
  <Icon icon="ph:plus" />
</button>
