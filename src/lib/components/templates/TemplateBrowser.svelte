<script lang="ts">
  import Icon from "@iconify/svelte";
  import { onMount } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import TemplateCard from "./TemplateCard.svelte";
  import {
    categoryCounts,
    categoryLabel,
    fetchTemplates,
    matchesQuery,
    sortTemplates,
    type TemplateSort,
    type UniverseTemplate,
  } from "#lib/ts/templates";

  let { onselect }: { onselect: (template: UniverseTemplate) => void } = $props();

  const PAGE_SIZE = 48;

  let templates = $state<UniverseTemplate[]>([]);
  let loading = $state(true);
  let loadError = $state("");
  let query = $state("");
  let category = $state("all");
  let sort = $state<TemplateSort>("name");
  let visibleCount = $state(PAGE_SIZE);

  let categoryOptions = $derived<[string, number][]>([["all", templates.length], ...categoryCounts(templates)]);
  let filtered = $derived(
    sortTemplates(
      templates.filter(
        (template) => (category === "all" || template.categories.includes(category)) && matchesQuery(template, query),
      ),
      sort,
    ),
  );
  let visible = $derived(filtered.slice(0, visibleCount));

  $effect(() => {
    query;
    category;
    sort;
    visibleCount = PAGE_SIZE;
  });

  async function load() {
    loading = true;
    loadError = "";
    try {
      templates = await fetchTemplates();
    } catch (error) {
      loadError = String(error);
    }
    loading = false;
  }

  onMount(load);
</script>

<div class="flex flex-col gap-4">
  <div class="flex flex-col gap-2 sm:flex-row sm:items-center">
    <label class="relative flex-1">
      <Icon icon="ph:magnifying-glass" class="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-sm text-[var(--color-ink-subtle)]" />
      <input
        bind:value={query}
        type="search"
        placeholder="Search templates, such as thesis, IEEE or CV"
        aria-label="Search templates"
        class="field w-full pl-8 text-sm"
      />
    </label>
    <select bind:value={sort} aria-label="Sort templates" class="field text-sm">
      <option value="name">By name</option>
      <option value="recent">Recently updated</option>
    </select>
  </div>

  {#if categoryOptions.length > 1}
    <div class="flex flex-wrap gap-1.5" role="group" aria-label="Template type">
      {#each categoryOptions as [option, count] (option)}
        <button
          onclick={() => (category = option)}
          aria-pressed={category === option}
          class="rounded-full border px-2.5 py-0.5 text-[11px] transition-colors {category === option
            ? 'border-[var(--color-accent)] bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
            : 'border-[var(--color-line)] text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
        >
          {option === "all" ? "All" : categoryLabel(option)}
          <span class="opacity-60">{count}</span>
        </button>
      {/each}
    </div>
  {/if}

  {#if loading}
    <p class="flex items-center gap-2 py-10 text-sm text-[var(--color-ink-muted)]">
      <Icon icon="ph:spinner" class="animate-spin" /> Loading templates from Typst Universe...
    </p>
  {:else if loadError}
    <div class="flex flex-col items-start gap-3 py-10">
      <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
      <button class="btn btn-secondary" onclick={load}>Try again</button>
    </div>
  {:else if filtered.length === 0}
    <p class="py-10 text-sm text-[var(--color-ink-muted)]">No templates match your search.</p>
  {:else}
    <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6">
      {#each visible as template (template.name)}
        <TemplateCard {template} onselect={() => onselect(template)} />
      {/each}
    </div>
    {#if visible.length < filtered.length}
      <button class="btn btn-secondary self-center" onclick={() => (visibleCount += PAGE_SIZE)}>
        Show more ({filtered.length - visible.length} left)
      </button>
    {/if}
  {/if}

  <p class="text-[11px] text-[var(--color-ink-muted)]">
    Templates come from
    <button class="text-[var(--color-accent)] hover:underline" onclick={() => openUrl("https://typst.app/universe")}>Typst Universe</button>
    and are made by the community. Each one is downloaded only when you use it.
  </p>
</div>
