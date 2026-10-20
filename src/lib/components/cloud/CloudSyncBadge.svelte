<script lang="ts">
  import Icon from "@iconify/svelte";

  let { syncState }: { syncState: "synced" | "pending" | null } = $props();
</script>

{#if syncState}
  <span
    class="flex shrink-0 items-center gap-1 rounded bg-[var(--color-surface)]/90 px-1.5 py-0.5 text-[11px] font-medium ring-1 ring-[var(--color-line)] {syncState === 'pending'
      ? 'text-[var(--color-accent)]'
      : 'text-[var(--color-success)]'}"
    title={syncState === "pending" ? "Local changes not yet synced" : "On this device and up to date"}
  >
    <Icon icon={syncState === "pending" ? "mdi:cloud-upload" : "mdi:laptop"} class="text-xs" />
    {syncState === "pending" ? "Unsynced" : "On device"}
  </span>
{/if}
