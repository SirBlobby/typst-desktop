<script lang="ts">
  import type { EditorView } from "@codemirror/view";
  import Modal from "../Modal.svelte";
  import AddReferencePanel from "./AddReferencePanel.svelte";
  import ZoteroSyncPanel from "./ZoteroSyncPanel.svelte";

  interface Props {
    path: string;
    latex: boolean;
    view: EditorView | null;
    onchanged: () => void;
    onopensettings: () => void;
    onclose: () => void;
  }

  let { path, latex, view, onchanged, onopensettings, onclose }: Props = $props();

  const tabs = [
    { id: "add", label: "Add reference" },
    { id: "zotero", label: "Zotero" },
  ] as const;

  let activeTab = $state<"add" | "zotero">("add");
</script>

<Modal title="Citations" icon="ph:quotes" width="max-w-xl" {onclose}>
  <div class="flex flex-col gap-4">
    <div class="flex gap-1 rounded-lg bg-[var(--color-surface-sunken)] p-1" role="tablist">
      {#each tabs as tab (tab.id)}
        <button
          role="tab"
          aria-selected={activeTab === tab.id}
          onclick={() => (activeTab = tab.id)}
          class="flex-1 rounded-md px-3 py-1.5 text-xs font-medium transition-colors
            {activeTab === tab.id
            ? 'bg-[var(--color-surface-raised)] text-[var(--color-ink)] shadow-sm'
            : 'text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
        >
          {tab.label}
        </button>
      {/each}
    </div>

    {#if activeTab === "add"}
      <AddReferencePanel {path} {latex} {view} {onchanged} />
    {:else}
      <ZoteroSyncPanel {path} {onchanged} {onopensettings} />
    {/if}
  </div>

  {#snippet footer()}
    <button class="btn btn-primary" onclick={onclose}>Done</button>
  {/snippet}
</Modal>
