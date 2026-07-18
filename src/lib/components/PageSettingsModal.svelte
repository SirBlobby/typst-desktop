<script lang="ts">
  import type { EditorView } from "@codemirror/view";
  import Modal from "./Modal.svelte";
  import { setTypstConfig } from "$lib/ts/editor-actions";

  interface Props {
    view: EditorView | null;
    onclose: () => void;
  }

  let { view, onclose }: Props = $props();

  const papers = [
    "a4",
    "a3",
    "a5",
    "us-letter",
    "us-legal",
    "presentation-16-9",
  ];

  let paper = $state("a4");
  let margin = $state("2.5cm");
  let columns = $state(1);
  let flipped = $state(false);
  let numbering = $state("none");

  function apply() {
    setTypstConfig(view, "page", "paper", `"${paper}"`);
    if (margin.trim()) {
      setTypstConfig(view, "page", "margin", margin.trim());
    }
    if (columns > 1) {
      setTypstConfig(view, "page", "columns", String(columns));
    }
    if (flipped) {
      setTypstConfig(view, "page", "flipped", "true");
    }
    if (numbering !== "none") {
      setTypstConfig(view, "page", "numbering", `"${numbering}"`);
    }
    onclose();
  }
</script>

<Modal title="Page settings" icon="ph:file-text" {onclose}>
  <div class="flex flex-col gap-3">
    <label class="flex flex-col gap-1 text-xs">
      <span class="font-medium text-[var(--color-ink-muted)]">Paper</span>
      <select
        class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none"
        bind:value={paper}
      >
        {#each papers as option}
          <option value={option}>{option}</option>
        {/each}
      </select>
    </label>

    <label class="flex flex-col gap-1 text-xs">
      <span class="font-medium text-[var(--color-ink-muted)]">Margin</span>
      <input
        class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none"
        bind:value={margin}
        placeholder="2.5cm"
      />
    </label>

    <label class="flex flex-col gap-1 text-xs">
      <span class="font-medium text-[var(--color-ink-muted)]">Columns</span>
      <input
        type="number"
        min="1"
        max="6"
        class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none"
        bind:value={columns}
      />
    </label>

    <label class="flex flex-col gap-1 text-xs">
      <span class="font-medium text-[var(--color-ink-muted)]">Page numbers</span>
      <select
        class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none"
        bind:value={numbering}
      >
        <option value="none">None</option>
        <option value="1">1, 2, 3</option>
        <option value="1 / 1">1 / 10</option>
        <option value="i">i, ii, iii</option>
      </select>
    </label>

    <label class="flex items-center gap-2 text-xs">
      <input type="checkbox" bind:checked={flipped} />
      <span>Landscape</span>
    </label>
  </div>

  {#snippet footer()}
    <button
      class="rounded-md px-3 py-1.5 text-xs text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]"
      onclick={onclose}
    >
      Cancel
    </button>
    <button
      class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90"
      onclick={apply}
    >
      Apply
    </button>
  {/snippet}
</Modal>
