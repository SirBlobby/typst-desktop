<script lang="ts">
  import { onMount } from "svelte";
  import Icon from "@iconify/svelte";
  import BibliographyFilePicker from "./BibliographyFilePicker.svelte";
  import {
    bibliographyFiles,
    citationSyncs,
    collectionLabels,
    createCitationSync,
    deleteCitationSync,
    runCitationSync,
    zoteroCollections,
    zoteroLibraries,
    zoteroStatus,
    type CitationSync,
    type ZoteroLibrary,
    type ZoteroStatus,
  } from "#lib/ts/citations";

  interface Props {
    path: string;
    onchanged: () => void;
    onopensettings: () => void;
  }

  let { path, onchanged, onopensettings }: Props = $props();

  let status = $state<ZoteroStatus | null>(null);
  let syncs = $state<CitationSync[]>([]);
  let files = $state<string[]>([]);
  let libraries = $state<ZoteroLibrary[]>([]);
  let collections = $state<{ key: string; label: string }[]>([]);
  let libraryId = $state("");
  let collectionKey = $state("");
  let fileName = $state("zotero.bib");
  let busyId = $state<string | null>(null);
  let error = $state("");

  const syncedFiles = $derived(new Set(syncs.map((sync) => sync.file_name)));
  const replacesExisting = $derived(files.includes(fileName) && !syncedFiles.has(fileName));

  function syncedLabel(value: string): string {
    const date = new Date(value.replace(" ", "T") + "Z");
    return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
  }

  onMount(async () => {
    try {
      status = await zoteroStatus();
      [syncs, files] = await Promise.all([citationSyncs(path), bibliographyFiles(path)]);
      if (status.connected) {
        libraries = await zoteroLibraries();
        libraryId = libraries[0]?.id ?? "";
      }
    } catch (err) {
      error = String(err);
    }
  });

  $effect(() => {
    const library = libraryId;
    collections = [];
    collectionKey = "";
    if (!library) return;
    zoteroCollections(library)
      .then((list) => {
        if (libraryId === library) collections = collectionLabels(list);
      })
      .catch((err) => (error = String(err)));
  });

  async function create() {
    const library = libraries.find((entry) => entry.id === libraryId);
    if (!library) return;
    busyId = "new";
    error = "";
    try {
      const collection = collections.find((entry) => entry.key === collectionKey);
      const sync = await createCitationSync(path, {
        libraryId: library.id,
        libraryName: library.name,
        collectionKey: collection?.key ?? null,
        collectionName: collection?.label ?? null,
        fileName,
      });
      syncs = [...syncs, sync];
      if (!files.includes(sync.file_name)) files = [...files, sync.file_name].sort();
      onchanged();
    } catch (err) {
      error = String(err);
    }
    busyId = null;
  }

  async function syncNow(sync: CitationSync) {
    busyId = sync.id;
    error = "";
    try {
      const updated = await runCitationSync(sync.id);
      syncs = syncs.map((entry) => (entry.id === updated.id ? updated : entry));
      onchanged();
    } catch (err) {
      error = String(err);
    }
    busyId = null;
  }

  async function remove(sync: CitationSync) {
    busyId = sync.id;
    error = "";
    try {
      await deleteCitationSync(sync.id);
      syncs = syncs.filter((entry) => entry.id !== sync.id);
    } catch (err) {
      error = String(err);
    }
    busyId = null;
  }
</script>

{#if !status}
  <p class="text-xs text-[var(--color-ink-muted)]">Loading...</p>
{:else}
  <div class="flex flex-col gap-4">
    {#if syncs.length > 0}
      <div class="card divide-y divide-[var(--color-line)] overflow-hidden">
        {#each syncs as sync (sync.id)}
          <div class="flex items-center gap-3 px-3 py-2.5">
            <Icon icon="ph:arrows-clockwise" class="shrink-0 text-lg text-[var(--color-ink-muted)]" />
            <div class="min-w-0 flex-1">
              <p class="truncate font-mono text-[13px]">{sync.file_name}</p>
              <p class="truncate text-xs text-[var(--color-ink-muted)]">
                {sync.collection_name ?? sync.library_name} · {sync.item_count} {sync.item_count === 1 ? "entry" : "entries"}
                {#if sync.last_synced_at}· Synced {syncedLabel(sync.last_synced_at)}{/if}
              </p>
            </div>
            <button class="icon-btn" disabled={busyId !== null || !status.connected} onclick={() => syncNow(sync)} title="Sync now" aria-label="Sync now">
              <Icon icon={busyId === sync.id ? "ph:spinner" : "ph:arrow-clockwise"} class={busyId === sync.id ? "animate-spin" : ""} />
            </button>
            <button class="icon-btn" disabled={busyId !== null} onclick={() => remove(sync)} title="Stop syncing" aria-label="Stop syncing">
              <Icon icon="ph:link-break" />
            </button>
          </div>
        {/each}
      </div>
    {/if}

    {#if !status.connected}
      <div class="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-3 text-xs text-[var(--color-ink-muted)]">
        Connect your Zotero account to sync a library or collection into a bibliography file.
        <button class="font-semibold text-[var(--color-accent)] hover:underline" onclick={onopensettings}>Open citation settings</button>
      </div>
    {:else}
      <div class="flex flex-col gap-3 rounded-lg border border-[var(--color-line)] p-3">
        <p class="text-xs font-medium">Sync from {status.username}'s Zotero</p>
        <div class="grid grid-cols-2 gap-3">
          <label class="flex flex-col gap-1.5 text-xs">
            <span class="font-medium text-[var(--color-ink-muted)]">Library</span>
            <select bind:value={libraryId} class="field text-sm">
              {#each libraries as library (library.id)}
                <option value={library.id}>{library.name}</option>
              {/each}
            </select>
          </label>
          <label class="flex flex-col gap-1.5 text-xs">
            <span class="font-medium text-[var(--color-ink-muted)]">Collection</span>
            <select bind:value={collectionKey} class="field text-sm">
              <option value="">Whole library</option>
              {#each collections as collection (collection.key)}
                <option value={collection.key}>{collection.label}</option>
              {/each}
            </select>
          </label>
        </div>
        <BibliographyFilePicker
          files={files.filter((file) => !syncedFiles.has(file))}
          bind:value={fileName}
          defaultName="zotero.bib"
          label="Save to"
          preferExisting={false}
        />
        {#if replacesExisting}
          <p class="text-xs text-amber-600 dark:text-amber-400">Everything in {fileName} will be replaced by the synced entries.</p>
        {/if}
        <div class="flex justify-end">
          <button class="btn btn-primary" disabled={busyId !== null || !libraryId || !fileName || syncedFiles.has(fileName)} onclick={create}>
            <Icon icon={busyId === "new" ? "ph:spinner" : "ph:arrows-clockwise"} class={busyId === "new" ? "animate-spin" : ""} />
            Sync to {fileName}
          </button>
        </div>
      </div>
    {/if}

    {#if error}
      <p class="text-xs text-[var(--color-danger)]">{error}</p>
    {/if}
  </div>
{/if}
