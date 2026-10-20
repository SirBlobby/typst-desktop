<script lang="ts">
  import Icon from "@iconify/svelte";
  import { editorViewStore } from "$lib/ts/review/session";
  import { focusOnMount } from "$lib/ts/focus-on-mount";
  import { submitSuggestion, suggestionDraftStore, type SuggestionDraft } from "$lib/ts/review/suggestions";

  let { docId }: { docId: string } = $props();

  let replacement = $state("");
  let note = $state("");
  let saving = $state(false);
  let errorMessage = $state("");
  let position = $state({ top: 120, left: 120 });
  let currentDraftKey = "";

  function draftKey(draft: SuggestionDraft) {
    return `${draft.anchor_start}:${draft.anchor_end}:${draft.replacement_text}`;
  }

  $effect(() => {
    const draft = $suggestionDraftStore;
    const view = $editorViewStore;
    if (!draft || draftKey(draft) === currentDraftKey) return;
    currentDraftKey = draftKey(draft);
    replacement = draft.replacement_text;
    note = "";
    errorMessage = "";
    const coordinates = view?.coordsAtPos(Math.min(draft.from, view.state.doc.length));
    if (coordinates) {
      position = {
        top: Math.min(coordinates.bottom + 8, window.innerHeight - 320),
        left: Math.min(Math.max(coordinates.left, 12), window.innerWidth - 372)
      };
    }
  });

  function close() {
    suggestionDraftStore.set(null);
    currentDraftKey = "";
    $editorViewStore?.focus();
  }

  async function submit() {
    const draft = $suggestionDraftStore;
    if (!draft || saving) return;
    if (replacement === draft.original_text) {
      errorMessage = "Change the text to make a suggestion.";
      return;
    }
    saving = true;
    errorMessage = "";
    try {
      await submitSuggestion(docId, { ...draft, replacement_text: replacement }, note.trim());
      close();
    } catch (err) {
      errorMessage = (err as Error).message;
    }
    saving = false;
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") close();
    if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) submit();
  }
</script>

{#if $suggestionDraftStore}
  <div
    role="dialog"
    aria-label="Suggest an edit"
    tabindex="-1"
    onkeydown={handleKeydown}
    class="fixed z-[90] w-[360px] rounded-xl border border-[var(--color-line)] bg-[var(--color-surface-raised)] p-3 shadow-2xl"
    style="top: {position.top}px; left: {position.left}px;"
  >
    <div class="mb-2 flex items-center gap-2">
      <Icon icon="ph:note-pencil" class="text-base text-[var(--color-accent)]" />
      <p class="text-sm font-semibold text-[var(--color-ink)]">Suggest an edit</p>
      <button type="button" onclick={close} class="ml-auto rounded p-1 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]" aria-label="Cancel suggestion">
        <Icon icon="ph:x" class="text-sm" />
      </button>
    </div>

    {#if $suggestionDraftStore.original_text}
      <p class="section-label mb-1">Replace</p>
      <p class="mb-2 max-h-20 overflow-y-auto whitespace-pre-wrap break-words rounded-md bg-rose-500/10 px-2 py-1 font-mono text-xs text-rose-700 line-through dark:text-rose-300">{$suggestionDraftStore.original_text}</p>
    {/if}

    <p class="section-label mb-1">{$suggestionDraftStore.original_text ? 'With' : 'Insert'}</p>
    <textarea
      bind:value={replacement}
      use:focusOnMount
      rows="3"
      placeholder="Leave empty to suggest deleting the text"
      class="field field-sm mb-2 resize-y font-mono"
    ></textarea>

    <input
      bind:value={note}
      placeholder="Add a note (optional)"
      class="field field-sm mb-2"
    />

    {#if errorMessage}
      <p class="mb-2 text-xs text-[var(--color-danger)]">{errorMessage}</p>
    {/if}

    <div class="flex items-center justify-between">
      <p class="text-[10px] text-[var(--color-ink-muted)]">Ctrl+Enter to suggest</p>
      <div class="flex gap-2">
        <button type="button" onclick={close} class="rounded-md px-2.5 py-1 text-xs font-medium text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]">Cancel</button>
        <button type="button" onclick={submit} disabled={saving} class="btn btn-primary">
          <Icon icon={saving ? 'ph:circle-notch' : 'ph:paper-plane-right'} class="text-sm {saving ? 'animate-spin' : ''}" />
          Suggest
        </button>
      </div>
    </div>
  </div>
{/if}
