<script lang="ts">
  import Icon from "@iconify/svelte";

  let { syncState }: { syncState: "synced" | "pending" | null } = $props();
</script>

{#if syncState}
  <span
    class="chip h-5 shrink-0 px-1.5 text-[10px] {syncState === 'pending'
      ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
      : 'bg-[var(--color-success)]/12 text-[var(--color-success)]'}"
    title={syncState === "pending" ? "Local changes not yet synced" : "On this device and up to date"}
  >
    <Icon icon={syncState === "pending" ? "ph:cloud-arrow-up" : "ph:laptop"} class="text-xs" />
    {syncState === "pending" ? "Unsynced" : "On device"}
  </span>
{/if}
