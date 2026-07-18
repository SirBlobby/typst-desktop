<script lang="ts">
  import Icon from "@iconify/svelte";
  import Modal from "./Modal.svelte";
  import * as api from "$lib/ts/api";
  import type { Resource } from "$lib/ts/api";
  import { pickFiles } from "$lib/ts/import";
  import { app } from "$lib/ts/state.svelte";

  interface Props {
    oninsert?: (snippet: string) => void;
    onchanged?: () => void;
    onclose: () => void;
  }

  let { oninsert, onchanged, onclose }: Props = $props();

  let resources = $state<Resource[]>([]);
  let previews = $state<Record<string, string>>({});
  let query = $state("");
  let scope = $state<"all" | "project" | "shared">("all");
  let busy = $state(false);
  let error = $state("");
  let copied = $state<string | null>(null);

  const filtered = $derived(
    resources.filter((resource) => {
      if (scope !== "all" && resource.scope !== scope) return false;
      const needle = query.trim().toLowerCase();
      if (!needle) return true;
      return (
        resource.name.toLowerCase().includes(needle) ||
        resource.reference.toLowerCase().includes(needle) ||
        resource.font_families.some((family) =>
          family.toLowerCase().includes(needle),
        )
      );
    }),
  );

  async function refresh() {
    if (!app.target) return;
    try {
      resources = await api.listResources(app.target.path);
    } catch (caught) {
      error = api.errorMessage(caught);
    }
  }

  $effect(() => {
    refresh();
  });

  $effect(() => {
    const images = filtered.filter((resource) => resource.kind === "image");
    let cancelled = false;

    (async () => {
      for (const resource of images) {
        if (cancelled) return;
        if (previews[resource.path]) continue;
        try {
          const result = await api.thumbnail(resource.path);
          if (!cancelled && result.kind === "image") {
            previews[resource.path] = result.data;
          }
        } catch {
          continue;
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  });

  async function importInto(destination: "project" | "shared") {
    const sources = await pickFiles("assets");
    if (sources.length === 0) return;

    busy = true;
    error = "";
    try {
      if (destination === "shared") {
        await api.importAssets(sources);
      } else if (app.target) {
        await api.importIntoTarget(app.target.path, sources);
      }
      await refresh();
      onchanged?.();
    } catch (caught) {
      error = api.errorMessage(caught);
    } finally {
      busy = false;
    }
  }

  async function remove(resource: Resource) {
    try {
      if (resource.scope === "shared") {
        await api.deleteAsset(resource.name);
      } else {
        await api.deleteEntry(resource.path);
      }
      delete previews[resource.path];
      await refresh();
      onchanged?.();
    } catch (caught) {
      error = api.errorMessage(caught);
    }
  }

  function insert(resource: Resource) {
    if (resource.kind === "image") {
      oninsert?.(`#image("${resource.reference}")`);
    } else if (resource.font_families.length > 0) {
      oninsert?.(`#set text(font: "${resource.font_families[0]}")`);
    } else {
      oninsert?.(`"${resource.reference}"`);
    }
  }

  async function copyReference(value: string) {
    await navigator.clipboard.writeText(value);
    copied = value;
    setTimeout(() => (copied = null), 1200);
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  }

  const iconFor: Record<string, string> = {
    image: "ph:image",
    font: "ph:text-aa",
    file: "ph:file",
  };
</script>

<Modal title="Assets" icon="ph:images" width="max-w-3xl" {onclose}>
  <div class="flex flex-col gap-3">
    <div class="flex items-center gap-2">
      <div
        class="flex flex-1 items-center gap-2 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 focus-within:border-[var(--color-accent)]"
      >
        <Icon icon="ph:magnifying-glass" class="text-[var(--color-ink-muted)]" />
        <input
          class="min-w-0 flex-1 bg-transparent text-sm focus:outline-none"
          placeholder="Search images, fonts, and files"
          bind:value={query}
        />
        {#if query}
          <button
            class="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]"
            onclick={() => (query = "")}
            aria-label="Clear search"
          >
            <Icon icon="ph:x" />
          </button>
        {/if}
      </div>

      <div class="flex rounded-md bg-[var(--color-surface-sunken)] p-0.5 text-xs">
        {#each [["all", "All"], ["project", "This project"], ["shared", "Shared"]] as [value, label]}
          <button
            class="rounded px-2.5 py-1.5 transition
              {scope === value
              ? 'bg-[var(--color-surface)] font-medium shadow-sm'
              : 'text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
            onclick={() => (scope = value as "all" | "project" | "shared")}
          >
            {label}
          </button>
        {/each}
      </div>
    </div>

    {#if error}
      <p class="text-xs text-[var(--color-danger)]">{error}</p>
    {/if}

    {#if filtered.length === 0}
      <div
        class="flex flex-col items-center gap-3 rounded-lg border border-dashed border-[var(--color-line)] px-4 py-10 text-center"
      >
        <Icon icon="ph:image-square" class="text-4xl text-[var(--color-ink-muted)]" />
        <p class="text-xs text-[var(--color-ink-muted)]">
          {query ? "Nothing matches that search." : "No images or fonts yet."}
        </p>
      </div>
    {:else}
      <div
        class="scroll-thin grid max-h-96 grid-cols-[repeat(auto-fill,minmax(150px,1fr))] gap-2 overflow-y-auto"
      >
        {#each filtered as resource (resource.path)}
          <div
            class="group relative flex flex-col overflow-hidden rounded-lg border border-[var(--color-line)] transition hover:border-[var(--color-accent)]"
          >
            <button
              class="flex h-20 items-center justify-center overflow-hidden bg-[var(--color-surface-muted)]"
              onclick={() => insert(resource)}
              title="Insert into document"
            >
              {#if previews[resource.path]}
                <img
                  src={previews[resource.path]}
                  alt={resource.name}
                  class="h-full w-full object-contain"
                />
              {:else}
                <Icon
                  icon={iconFor[resource.kind] ?? "ph:file"}
                  class="text-2xl text-[var(--color-accent)]"
                />
              {/if}
            </button>

            <div class="flex flex-col gap-0.5 px-2 py-1.5">
              <span class="truncate text-[11px] font-medium" title={resource.reference}>
                {resource.name}
              </span>

              {#if resource.font_families.length > 0}
                <button
                  class="truncate text-left text-[10px] text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]"
                  onclick={() => copyReference(resource.font_families[0])}
                  title="Copy family name"
                >
                  {copied === resource.font_families[0]
                    ? "Copied"
                    : resource.font_families[0]}
                </button>
              {:else}
                <span class="text-[10px] text-[var(--color-ink-muted)]">
                  {resource.scope === "shared" ? "Shared" : "Project"} · {formatSize(
                    resource.size,
                  )}
                </span>
              {/if}
            </div>

            <button
              class="absolute right-1 top-1 rounded bg-[var(--color-surface)]/90 p-1 text-[var(--color-ink-muted)] opacity-0 transition group-hover:opacity-100 hover:text-[var(--color-danger)]"
              onclick={() => remove(resource)}
              aria-label="Delete"
            >
              <Icon icon="ph:trash" class="text-xs" />
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  {#snippet footer()}
    <button
      class="rounded-md px-3 py-1.5 text-xs text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]"
      onclick={onclose}
    >
      Close
    </button>
    <button
      class="flex items-center gap-1.5 rounded-md border border-[var(--color-line)] px-3 py-1.5 text-xs transition hover:bg-[var(--color-surface-muted)] disabled:opacity-50"
      disabled={busy}
      onclick={() => importInto("shared")}
    >
      <Icon icon="ph:upload-simple" />
      Add to shared
    </button>
    <button
      class="flex items-center gap-1.5 rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90 disabled:opacity-50"
      disabled={busy}
      onclick={() => importInto("project")}
    >
      <Icon icon="ph:upload-simple" />
      {busy ? "Importing..." : "Add to project"}
    </button>
  {/snippet}
</Modal>
