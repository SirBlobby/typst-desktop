<script lang="ts">
  import Icon from "@iconify/svelte";

  interface Props {
    thumbnail: { kind: string; data: string } | null;
    name: string;
    fallbackIcon: string;
    fallbackClass?: string;
  }

  let {
    thumbnail,
    name,
    fallbackIcon,
    fallbackClass = "bg-[var(--color-accent-soft)] text-[var(--color-accent)]",
  }: Props = $props();

  function svgDataUrl(svg: string): string {
    const bytes = new TextEncoder().encode(svg);
    let binary = "";
    for (const byte of bytes) binary += String.fromCharCode(byte);
    return `data:image/svg+xml;base64,${btoa(binary)}`;
  }

  const source = $derived(
    thumbnail?.kind === "svg" ? svgDataUrl(thumbnail.data) : (thumbnail?.data ?? null),
  );
</script>

<div
  class="pointer-events-none relative flex h-40 w-full items-center justify-center overflow-hidden border-b border-[var(--color-line)] bg-[var(--color-surface-muted)]"
>
  {#if thumbnail?.kind === "svg" && source}
    <div class="flex h-full w-full items-start justify-center bg-white transition-transform duration-300 group-hover:scale-105">
      <img src={source} alt={name} class="h-auto w-full" draggable="false" />
    </div>
  {:else if source}
    <img
      src={source}
      alt={name}
      class="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
      draggable="false"
    />
  {:else}
    <div class="rounded-xl p-3.5 transition-transform duration-300 group-hover:scale-105 {fallbackClass}">
      <Icon icon={fallbackIcon} class="text-3xl" />
    </div>
  {/if}
</div>
