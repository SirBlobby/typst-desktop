<script lang="ts">
  import Icon from "@iconify/svelte";
  import Modal from "./Modal.svelte";
  import * as api from "$lib/ts/api";
  import type { Asset } from "$lib/ts/api";
  import { pickFiles } from "$lib/ts/import";

  interface Props {
    oninsert?: (snippet: string) => void;
    onchanged?: () => void;
    onclose: () => void;
  }

  let { oninsert, onchanged, onclose }: Props = $props();

  let assets = $state<Asset[]>([]);
  let busy = $state(false);
  let error = $state("");
  let copied = $state<string | null>(null);

  async function refresh() {
    try {
      assets = await api.listAssets();
    } catch (caught) {
      error = api.errorMessage(caught);
    }
  }

  $effect(() => {
    refresh();
  });

  async function importFiles() {
    const sources = await pickFiles("assets");
    if (sources.length === 0) return;

    busy = true;
    error = "";
    try {
      await api.importAssets(sources);
      await refresh();
      onchanged?.();
    } catch (caught) {
      error = api.errorMessage(caught);
    } finally {
      busy = false;
    }
  }

  async function remove(asset: Asset) {
    try {
      await api.deleteAsset(asset.name);
      await refresh();
      onchanged?.();
    } catch (caught) {
      error = api.errorMessage(caught);
    }
  }

  function insert(asset: Asset) {
    if (asset.kind === "image") {
      oninsert?.(`#image("${asset.name}")`);
    } else if (asset.font_families.length > 0) {
      oninsert?.(`#set text(font: "${asset.font_families[0]}")`);
    }
  }

  async function copyFamily(family: string) {
    await navigator.clipboard.writeText(family);
    copied = family;
    setTimeout(() => (copied = null), 1200);
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  }

  const iconFor: Record<Asset["kind"], string> = {
    image: "ph:image",
    font: "ph:text-aa",
    file: "ph:file",
  };
</script>

<Modal title="Images and fonts" icon="ph:images" width="max-w-2xl" {onclose}>
  <div class="flex flex-col gap-4">
    <p class="text-xs text-[var(--color-ink-muted)]">
      Files imported here are available to every project. Reference an image by
      its file name, and a font by its family name.
    </p>

    {#if error}
      <p class="text-xs text-[var(--color-danger)]">{error}</p>
    {/if}

    {#if assets.length === 0}
      <div
        class="flex flex-col items-center gap-3 rounded-lg border border-dashed border-[var(--color-line)] px-4 py-10 text-center"
      >
        <Icon icon="ph:image-square" class="text-4xl text-[var(--color-ink-muted)]" />
        <p class="text-xs text-[var(--color-ink-muted)]">
          No images or fonts imported yet.
        </p>
      </div>
    {:else}
      <div class="flex flex-col gap-1">
        {#each assets as asset (asset.name)}
          <div
            class="group flex items-center gap-3 rounded-md border border-[var(--color-line)] px-3 py-2"
          >
            <Icon
              icon={iconFor[asset.kind]}
              class="text-lg text-[var(--color-accent)]"
            />

            <div class="min-w-0 flex-1">
              <p class="truncate text-xs font-medium">{asset.name}</p>
              {#if asset.font_families.length > 0}
                <div class="mt-0.5 flex flex-wrap gap-1">
                  {#each asset.font_families as family}
                    <button
                      class="rounded bg-[var(--color-surface-sunken)] px-1.5 py-px text-[10px] text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]"
                      onclick={() => copyFamily(family)}
                      title="Copy family name"
                    >
                      {copied === family ? "Copied" : family}
                    </button>
                  {/each}
                </div>
              {:else}
                <p class="text-[10px] text-[var(--color-ink-muted)]">
                  {formatSize(asset.size)}
                </p>
              {/if}
            </div>

            {#if oninsert && (asset.kind === "image" || asset.font_families.length > 0)}
              <button
                class="rounded border border-[var(--color-line)] px-2 py-1 text-[10px] opacity-0 transition group-hover:opacity-100 hover:bg-[var(--color-surface-muted)]"
                onclick={() => insert(asset)}
              >
                Insert
              </button>
            {/if}

            <button
              class="rounded p-1 text-[var(--color-ink-muted)] opacity-0 transition group-hover:opacity-100 hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-danger)]"
              onclick={() => remove(asset)}
              aria-label="Delete"
            >
              <Icon icon="ph:trash" />
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
      class="flex items-center gap-1.5 rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90 disabled:opacity-50"
      disabled={busy}
      onclick={importFiles}
    >
      <Icon icon="ph:upload-simple" />
      {busy ? "Importing..." : "Import files"}
    </button>
  {/snippet}
</Modal>
