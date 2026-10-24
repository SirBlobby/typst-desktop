<script lang="ts">
  import Icon from "@iconify/svelte";
  import { untrack } from "svelte";
  import Modal from "./Modal.svelte";
  import { focusOnMount } from "$lib/ts/focus-on-mount";

  interface Props {
    existingPaths: string[];
    folders: string[];
    initialFolder?: string;
    defaultExtension: string;
    onsubmit: (path: string) => void;
    onclose: () => void;
  }

  let {
    existingPaths,
    folders,
    initialFolder = "",
    defaultExtension,
    onsubmit,
    onclose,
  }: Props = $props();

  const fileTypes = [
    { extension: "typ", label: "Typst", icon: "ph:file-text" },
    { extension: "tex", label: "LaTeX", icon: "ph:function" },
    { extension: "bib", label: "Bibliography", icon: "ph:book-open" },
    { extension: "md", label: "Markdown", icon: "ph:markdown-logo" },
    { extension: "txt", label: "Text", icon: "ph:file" },
  ];

  let folder = $state(untrack(() => initialFolder));
  let name = $state("");
  let extension = $state(untrack(() => defaultExtension));

  const typedExtension = $derived(
    name.trim().match(/\.([a-z0-9]+)$/i)?.[1].toLowerCase() ?? null,
  );
  const baseName = $derived(
    typedExtension ? name.trim().slice(0, -(typedExtension.length + 1)) : name.trim(),
  );
  const fileName = $derived(`${baseName}.${typedExtension ?? extension}`);
  const path = $derived(folder ? `${folder}/${fileName}` : fileName);

  const problem = $derived.by(() => {
    if (!baseName) return "Enter a file name.";
    if (/[\\/:*?"<>|]/.test(baseName)) return 'File names cannot contain / \\ : * ? " < > or |.';
    if (existingPaths.some((existing) => existing.toLowerCase() === path.toLowerCase())) {
      return `${path} already exists.`;
    }
    return null;
  });

  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (problem) return;
    onsubmit(path);
  }
</script>

<Modal title="New file" icon="ph:file-plus" {onclose}>
  <form id="new-file-form" onsubmit={submit} class="flex flex-col gap-4">
    <div class="flex flex-col gap-1.5 text-xs">
      <span class="font-medium text-[var(--color-ink-muted)]">Type</span>
      <div class="grid grid-cols-5 gap-2" role="radiogroup" aria-label="File type">
        {#each fileTypes as type (type.extension)}
          <button
            type="button"
            role="radio"
            aria-checked={extension === type.extension}
            onclick={() => (extension = type.extension)}
            class="flex flex-col items-center gap-1 rounded-lg border px-1 py-2 text-[11px] transition-colors
              {extension === type.extension
              ? 'border-[var(--color-accent)] bg-[var(--color-accent-soft)] text-[var(--color-ink)]'
              : 'border-[var(--color-line)] text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]'}"
          >
            <Icon
              icon={type.icon}
              class="text-lg {extension === type.extension ? 'text-[var(--color-accent)]' : ''}"
            />
            {type.label}
          </button>
        {/each}
      </div>
    </div>

    <label class="flex flex-col gap-1.5 text-xs">
      <span class="font-medium text-[var(--color-ink-muted)]">Name</span>
      <div class="field flex items-center p-0">
        <input
          use:focusOnMount
          bind:value={name}
          placeholder="chapter"
          class="min-w-0 flex-1 bg-transparent px-3 py-2 text-sm focus:outline-none"
        />
        {#if !typedExtension}
          <span class="pr-3 text-sm text-[var(--color-ink-muted)]">.{extension}</span>
        {/if}
      </div>
    </label>

    {#if folders.length > 0}
      <label class="flex flex-col gap-1.5 text-xs">
        <span class="font-medium text-[var(--color-ink-muted)]">Folder</span>
        <select bind:value={folder} class="field text-sm">
          <option value="">Project root</option>
          {#each folders as option (option)}
            <option value={option}>{option}</option>
          {/each}
        </select>
      </label>
    {/if}

    {#if problem && baseName}
      <p class="text-xs text-[var(--color-danger)]">{problem}</p>
    {/if}
  </form>

  {#snippet footer()}
    <button class="btn btn-ghost" onclick={onclose}>Cancel</button>
    <button type="submit" form="new-file-form" class="btn btn-primary" disabled={problem !== null}>
      Create
    </button>
  {/snippet}
</Modal>
