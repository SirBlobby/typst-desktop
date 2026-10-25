<script lang="ts">
  import Icon from "@iconify/svelte";
  import Modal from "#lib/components/Modal.svelte";
  import { focusOnMount } from "#lib/ts/focus-on-mount";
  import * as api from "#lib/ts/api";
  import type { FileEntry } from "#lib/ts/api";

  let { targetPath, files, activePath, onInsert, onClose }: {
    targetPath: string;
    files: FileEntry[];
    activePath: string | null;
    onInsert: (code: string) => void;
    onClose: () => void;
  } = $props();

  let searchQuery = $state("");
  let loadError = $state("");
  let insertingPath = $state<string | null>(null);

  let visibleFiles = $derived(
    files.filter(
      (file) =>
        !file.is_dir &&
        file.path !== activePath &&
        file.path.toLowerCase().endsWith(".typ") &&
        file.path.toLowerCase().includes(searchQuery.trim().toLowerCase()),
    ),
  );

  function insertInclude(file: FileEntry) {
    onInsert(`#include "${file.path}"`);
  }

  async function insertContent(file: FileEntry) {
    insertingPath = file.path;
    try {
      const payload = await api.readTargetFile(targetPath, file.path);
      onInsert(payload.content);
    } catch (error) {
      loadError = api.errorMessage(error);
    }
    insertingPath = null;
  }
</script>

<Modal title="Insert document" icon="ph:file-text" width="max-w-lg" onclose={onClose}>
  <div class="flex flex-col gap-3">
    <p class="text-xs text-[var(--color-ink-muted)]">Include another file of this project, or copy its content at the cursor.</p>
    <div class="relative">
      <Icon icon="ph:magnifying-glass" class="absolute left-3 top-1/2 -translate-y-1/2 text-lg text-[var(--color-ink-muted)]" />
      <input
        type="search"
        bind:value={searchQuery}
        placeholder="Search files"
        use:focusOnMount
        class="field pl-9"
      />
    </div>

    {#if loadError}
      <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
    {/if}

    {#if visibleFiles.length === 0}
      <p class="py-6 text-center text-sm text-[var(--color-ink-muted)]">
        {searchQuery.trim() ? "No files match your search." : "This project has no other Typst files."}
      </p>
    {:else}
      <ul class="scroll-thin -mx-2 max-h-80 overflow-y-auto">
        {#each visibleFiles as file (file.path)}
          <li class="flex items-center gap-2 rounded-md px-2 py-1.5 hover:bg-[var(--color-surface-sunken)]">
            <Icon icon="ph:file-text" class="shrink-0 text-lg text-[var(--color-accent)]" />
            <span class="min-w-0 flex-1 truncate text-sm" title={file.path}>{file.path}</span>
            <button
              type="button"
              onclick={() => insertInclude(file)}
              class="btn btn-secondary h-6 px-2"
            >
              Include
            </button>
            <button
              type="button"
              onclick={() => insertContent(file)}
              disabled={insertingPath !== null}
              class="btn btn-secondary h-6 gap-1 px-2"
            >
              {#if insertingPath === file.path}
                <Icon icon="ph:circle-notch" class="animate-spin" />
              {/if}
              Copy content
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</Modal>
