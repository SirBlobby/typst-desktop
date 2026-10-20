<script lang="ts">
  import Icon from "@iconify/svelte";
  import { sortOptions, type CloudFilters, type SortField, type ViewMode } from "$lib/ts/cloud-entries";

  let {
    filters = $bindable(),
    viewMode = $bindable(),
  }: { filters: CloudFilters; viewMode: ViewMode } = $props();

  const layoutOptions: { value: ViewMode; icon: string; label: string }[] = [
    { value: "grid", icon: "mdi:view-grid-outline", label: "Grid view" },
    { value: "list", icon: "mdi:view-list-outline", label: "List view" },
  ];

  function toggleSortDirection() {
    filters = { ...filters, sortDirection: filters.sortDirection === "asc" ? "desc" : "asc" };
  }
</script>

<div class="mb-6 flex flex-wrap items-center justify-end gap-2">
  <label class="relative inline-flex cursor-pointer items-center gap-1.5 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] py-1.5 pl-3 pr-8 text-sm text-[var(--color-ink-muted)] transition-colors hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]">
    <Icon icon="mdi:sort" class="shrink-0 text-base" />
    <span class="sr-only">Sort by</span>
    <select
      value={filters.sortField}
      onchange={(event) => (filters = { ...filters, sortField: event.currentTarget.value as SortField })}
      aria-label="Sort by"
      class="cursor-pointer appearance-none bg-transparent font-medium outline-none"
    >
      {#each sortOptions as option (option.value)}
        <option value={option.value} class="bg-[var(--color-surface)] text-[var(--color-ink)]">{option.label}</option>
      {/each}
    </select>
    <Icon icon="mdi:chevron-down" class="pointer-events-none absolute right-2.5 text-base" />
  </label>

  <button
    class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] p-2 text-[var(--color-ink-muted)] transition-colors hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]"
    title={filters.sortDirection === "asc" ? "Ascending" : "Descending"}
    aria-label={filters.sortDirection === "asc" ? "Sort ascending" : "Sort descending"}
    onclick={toggleSortDirection}
  >
    <Icon icon={filters.sortDirection === "asc" ? "mdi:sort-ascending" : "mdi:sort-descending"} class="text-base" />
  </button>

  <div class="flex rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] p-0.5" role="radiogroup" aria-label="Layout">
    {#each layoutOptions as option (option.value)}
      <button
        role="radio"
        aria-checked={viewMode === option.value}
        aria-label={option.label}
        title={option.label}
        class="rounded-md p-1.5 transition-colors {viewMode === option.value
          ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
          : 'text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
        onclick={() => (viewMode = option.value)}
      >
        <Icon icon={option.icon} class="text-base" />
      </button>
    {/each}
  </div>
</div>
