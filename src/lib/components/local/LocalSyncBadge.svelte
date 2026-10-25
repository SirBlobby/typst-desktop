<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { BrowseEntry } from "#lib/ts/api";
  import { syncLabel } from "#lib/ts/local-entries";

  let { entry }: { entry: BrowseEntry } = $props();
</script>

{#if entry.cloud_linked}
  <span class="flex shrink-0 items-center" title={syncLabel(entry)}>
    {#if entry.sync_state === "pending"}
      <Icon icon="ph:cloud-arrow-up" class="text-sm text-[var(--color-accent)]" />
    {:else}
      <Icon icon="ph:cloud-check" class="text-sm text-[var(--color-success)]" />
    {/if}
  </span>
{/if}
