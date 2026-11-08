<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { UniverseTemplate } from "#lib/ts/templates";

  let { template, class: className = "" }: { template: UniverseTemplate; class?: string } = $props();

  let failed = $state(false);
</script>

<div class="flex items-start justify-center overflow-hidden bg-[var(--color-surface-sunken)] {className}">
  {#if template.thumbnail_url && !failed}
    <img
      src={template.thumbnail_url}
      alt="Preview of {template.name}"
      loading="lazy"
      decoding="async"
      class="h-full w-full object-cover object-top"
      onerror={() => (failed = true)}
    />
  {:else}
    <div class="flex h-full w-full items-center justify-center text-[var(--color-ink-muted)]">
      <Icon icon="ph:file-text" class="text-4xl opacity-60" />
    </div>
  {/if}
</div>
