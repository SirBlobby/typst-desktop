<script lang="ts">
  import Icon from "@iconify/svelte";

  let { title, description, icon }: { title: string; description?: string; icon: string } = $props();
</script>

<header class="mb-4 flex items-start gap-3 border-b border-[var(--color-line)] pb-4">
  <span class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-[var(--color-accent-soft)] text-[var(--color-accent)]">
    <Icon {icon} class="text-lg" />
  </span>
  <div class="min-w-0">
    <h2 class="text-sm font-semibold">{title}</h2>
    {#if description}
      <p class="mt-0.5 text-xs text-[var(--color-ink-muted)]">{description}</p>
    {/if}
  </div>
</header>
