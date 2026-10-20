<script lang="ts">
  import Icon from "@iconify/svelte";

  let { title, description, icon }: { title: string; description?: string; icon: string } = $props();
</script>

<header class="mb-5 flex items-start gap-3">
  <span class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-[var(--color-accent-soft)] text-[var(--color-accent)]">
    <Icon {icon} class="text-lg" />
  </span>
  <div class="min-w-0">
    <h2 class="text-base font-semibold tracking-tight">{title}</h2>
    {#if description}
      <p class="mt-0.5 text-xs text-[var(--color-ink-muted)]">{description}</p>
    {/if}
  </div>
</header>
