<script lang="ts">
  import { onMount } from "svelte";
  import Icon from "@iconify/svelte";
  import type { EditorView } from "@codemirror/view";
  import BibliographyFilePicker from "./BibliographyFilePicker.svelte";
  import { insertText } from "#lib/ts/editor-actions";
  import {
    addCitation,
    bibliographyFiles,
    citationMarkup,
    formatForFile,
    lookupCitation,
    type AddedEntry,
    type LookupResult,
  } from "#lib/ts/citations";

  interface Props {
    path: string;
    latex: boolean;
    view: EditorView | null;
    onchanged: () => void;
  }

  let { path, latex, view, onchanged }: Props = $props();

  let files = $state<string[]>([]);
  let fileName = $state("references.bib");
  let identifier = $state("");
  let preview = $state<LookupResult | null>(null);
  let added = $state<AddedEntry | null>(null);
  let busy = $state<"lookup" | "add" | null>(null);
  let error = $state("");
  let copied = $state(false);

  const format = $derived(formatForFile(fileName));

  onMount(async () => {
    try {
      files = await bibliographyFiles(path);
    } catch (err) {
      error = String(err);
    }
  });

  async function lookUp(event?: SubmitEvent) {
    event?.preventDefault();
    if (!identifier.trim()) return;
    busy = "lookup";
    error = "";
    added = null;
    try {
      preview = await lookupCitation(identifier, format);
    } catch (err) {
      preview = null;
      error = String(err);
    }
    busy = null;
  }

  async function addToFile() {
    busy = "add";
    error = "";
    try {
      added = await addCitation(path, fileName, identifier);
      if (!files.includes(added.file_name)) files = [...files, added.file_name].sort();
      preview = null;
      identifier = "";
      onchanged();
    } catch (err) {
      error = String(err);
    }
    busy = null;
  }

  async function copyEntry() {
    if (!preview) return;
    try {
      await navigator.clipboard.writeText(preview.entry);
      copied = true;
      setTimeout(() => (copied = false), 2000);
    } catch {}
  }

  function insertCitation(key: string) {
    if (!view) return;
    insertText(view, citationMarkup(key, latex));
    view.focus();
  }
</script>

<div class="flex flex-col gap-4">
  <form onsubmit={lookUp} class="flex flex-col gap-1.5 text-xs">
    <label for="citation-identifier" class="font-medium text-[var(--color-ink-muted)]">DOI, arXiv, PubMed, PMC or Semantic Scholar ID</label>
    <div class="flex gap-2">
      <input
        id="citation-identifier"
        bind:value={identifier}
        placeholder="10.1038/nature14539, 1706.03762, PMID 31452104"
        class="field min-w-0 flex-1 font-mono text-sm"
      />
      <button type="submit" class="btn btn-secondary" disabled={!identifier.trim() || busy !== null}>
        <Icon icon={busy === "lookup" ? "ph:spinner" : "ph:magnifying-glass"} class={busy === "lookup" ? "animate-spin" : ""} />
        Look up
      </button>
    </div>
    <p class="text-[var(--color-ink-muted)]">Paste an identifier or a link from doi.org, arxiv.org, pubmed.ncbi.nlm.nih.gov or semanticscholar.org.</p>
  </form>

  <BibliographyFilePicker {files} bind:value={fileName} defaultName="references.bib" />

  {#if error}
    <p class="text-xs text-[var(--color-danger)]">{error}</p>
  {/if}

  {#if preview}
    <div class="flex flex-col gap-2 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-3">
      <div class="flex items-start justify-between gap-3">
        <div class="min-w-0">
          <p class="text-sm font-medium">{preview.title}</p>
          <p class="font-mono text-xs text-[var(--color-ink-muted)]">{citationMarkup(preview.key, latex)}</p>
        </div>
        <button class="icon-btn" onclick={copyEntry} title="Copy entry" aria-label="Copy entry">
          <Icon icon={copied ? "ph:check" : "ph:copy"} />
        </button>
      </div>
      <pre class="scroll-thin max-h-56 overflow-auto rounded-md bg-[var(--color-surface-sunken)] p-2.5 font-mono text-[11px] leading-relaxed">{preview.entry}</pre>
      <div class="flex justify-end">
        <button class="btn btn-primary" onclick={addToFile} disabled={busy !== null || !fileName}>
          <Icon icon={busy === "add" ? "ph:spinner" : "ph:plus"} class={busy === "add" ? "animate-spin" : ""} />
          Add to {fileName}
        </button>
      </div>
    </div>
  {/if}

  {#if added}
    <div class="flex items-center gap-3 rounded-lg border border-[var(--color-success)]/40 bg-[var(--color-success)]/10 p-3">
      <Icon icon="ph:check-circle" class="shrink-0 text-xl text-[var(--color-success)]" />
      <p class="min-w-0 flex-1 text-xs">
        Added <span class="font-mono">{added.key}</span> to {added.file_name}.
        {#if added.created_file && !latex}
          Add <span class="font-mono">#bibliography("{added.file_name}")</span> where the list should appear.
        {/if}
      </p>
      {#if view}
        <button class="btn btn-secondary shrink-0" onclick={() => added && insertCitation(added.key)}>Insert citation</button>
      {/if}
    </div>
  {/if}
</div>
