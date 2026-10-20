<script lang="ts">
  import { onMount } from "svelte";
  import Icon from "@iconify/svelte";
  import Modal from "$lib/components/Modal.svelte";
  import { focusOnMount } from "$lib/ts/focus-on-mount";
  import * as api from "$lib/ts/api";
  import type { Resource } from "$lib/ts/api";

  let { targetPath, onInsert, onClose }: {
    targetPath: string;
    onInsert: (code: string) => void;
    onClose: () => void;
  } = $props();

  let images = $state<Resource[]>([]);
  let previews = $state<Record<string, string>>({});
  let loading = $state(true);
  let loadError = $state("");
  let searchQuery = $state("");
  let width = $state("100%");

  let visibleImages = $derived(
    images.filter((image) => image.reference.toLowerCase().includes(searchQuery.trim().toLowerCase())),
  );

  function insert(image: Resource) {
    const widthArgument = width.trim() ? `, width: ${width.trim()}` : "";
    onInsert(`#image("${image.reference}"${widthArgument})`);
  }

  async function loadPreviews() {
    for (const image of images) {
      try {
        const result = await api.thumbnail(image.path);
        if (result.kind === "image") previews[image.path] = result.data;
      } catch {
        continue;
      }
    }
  }

  onMount(async () => {
    try {
      const resources = await api.listResources(targetPath);
      images = resources.filter((resource) => resource.kind === "image");
      loadPreviews();
    } catch (error) {
      loadError = api.errorMessage(error);
    }
    loading = false;
  });
</script>

<Modal title="Insert image" icon="mdi:image-outline" width="max-w-2xl" onclose={onClose}>
  <div class="flex flex-col gap-3">
    <div class="flex gap-2">
      <div class="relative flex-1">
        <Icon icon="mdi:magnify" class="absolute left-3 top-1/2 -translate-y-1/2 text-lg text-[var(--color-ink-muted)]" />
        <input
          type="search"
          bind:value={searchQuery}
          placeholder="Search images in this project and your shared assets"
          use:focusOnMount
          class="w-full rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] py-2 pl-9 pr-3 text-sm focus:border-[var(--color-accent)] focus:outline-none"
        />
      </div>
      <label class="flex items-center gap-2 text-xs text-[var(--color-ink-muted)]">
        Width
        <input
          bind:value={width}
          placeholder="auto"
          class="w-20 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-2 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none"
        />
      </label>
    </div>

    {#if loading}
      <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading images...</p>
    {:else if loadError}
      <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
    {:else if visibleImages.length === 0}
      <p class="py-6 text-center text-sm text-[var(--color-ink-muted)]">
        {images.length === 0 ? "No images yet. Add some from Images and fonts." : "No images match your search."}
      </p>
    {:else}
      <div class="scroll-thin grid max-h-96 grid-cols-2 gap-3 overflow-y-auto sm:grid-cols-4">
        {#each visibleImages as image (image.path)}
          <button
            type="button"
            onclick={() => insert(image)}
            class="flex flex-col overflow-hidden rounded-md border border-[var(--color-line)] text-left transition hover:border-[var(--color-accent)]"
          >
            <div class="flex aspect-square items-center justify-center bg-[var(--color-surface-muted)]">
              {#if previews[image.path]}
                <img src={previews[image.path]} alt={image.name} class="max-h-full max-w-full object-contain" />
              {:else}
                <Icon icon="ph:image" class="text-3xl text-[var(--color-ink-muted)]" />
              {/if}
            </div>
            <div class="px-2 py-1.5">
              <p class="truncate text-xs font-medium" title={image.reference}>{image.name}</p>
              <p class="text-[10px] text-[var(--color-ink-muted)]">{image.scope === "project" ? "Project" : "Shared assets"}</p>
            </div>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</Modal>
