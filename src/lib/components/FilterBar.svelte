<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { SortField, SortOption, ViewFilters, ViewMode } from "#lib/ts/view-filters";

  let {
    filters = $bindable(),
    viewMode = $bindable(),
    sortOptions,
  }: { filters: ViewFilters; viewMode: ViewMode; sortOptions: SortOption[] } = $props();

  const layoutOptions: { value: ViewMode; icon: string; label: string }[] = [
    { value: "grid", icon: "ph:squares-four", label: "Grid view" },
    { value: "list", icon: "ph:list", label: "List view" },
  ];

  function toggleSortDirection() {
    filters = { ...filters, sortDirection: filters.sortDirection === "asc" ? "desc" : "asc" };
  }
</script>

<div class="mb-5 flex flex-wrap items-center justify-end gap-1.5">
  <label class="btn btn-secondary relative cursor-pointer pr-7 font-normal text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]">
    <Icon icon="ph:sort-ascending" class="shrink-0 text-sm" />
    <span class="sr-only">Sort by</span>
    <select
      value={filters.sortField}
      onchange={(event) => (filters = { ...filters, sortField: event.currentTarget.value as SortField })}
      aria-label="Sort by"
      class="cursor-pointer appearance-none bg-transparent font-medium outline-none"
    >
      {#each sortOptions as option (option.value)}
        <option value={option.value} class="bg-[var(--color-surface-raised)] text-[var(--color-ink)]">{option.label}</option>
      {/each}
    </select>
    <Icon icon="ph:caret-down" class="pointer-events-none absolute right-2 text-xs" />
  </label>

  <button
    class="btn btn-secondary w-7 px-0 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]"
    title={filters.sortDirection === "asc" ? "Ascending" : "Descending"}
    aria-label={filters.sortDirection === "asc" ? "Sort ascending" : "Sort descending"}
    onclick={toggleSortDirection}
  >
    <Icon icon={filters.sortDirection === "asc" ? "ph:arrow-up" : "ph:arrow-down"} class="text-sm" />
  </button>

  <div class="flex h-7 items-center rounded-md border border-[var(--color-line)] bg-[var(--color-surface-sunken)] p-0.5" role="radiogroup" aria-label="Layout">
    {#each layoutOptions as option (option.value)}
      <button
        role="radio"
        aria-checked={viewMode === option.value}
        aria-label={option.label}
        title={option.label}
        class="flex h-full w-7 items-center justify-center rounded transition-colors {viewMode === option.value
          ? 'bg-[var(--color-surface-raised)] text-[var(--color-ink)] shadow-sm'
          : 'text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
        onclick={() => (viewMode = option.value)}
      >
        <Icon icon={option.icon} class="text-sm" />
      </button>
    {/each}
  </div>
</div>
