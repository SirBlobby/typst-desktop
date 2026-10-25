<script lang="ts">
  import type { EditorView } from "@codemirror/view";
  import { app } from "#lib/ts/state.svelte";
  import { editorViewStore, reviewSessionStore } from "#lib/ts/review/session";
  import { historyOpenStore, reviewPanelOpenStore } from "#lib/ts/review/client";
  import { setLocalAwarenessUser } from "#lib/ts/review/awareness";
  import { resetComments } from "#lib/ts/review/comments";
  import { resetSuggestions, suggestingModeStore } from "#lib/ts/review/suggestions";
  import CommentHighlights from "./CommentHighlights.svelte";
  import SuggestionHighlights from "./SuggestionHighlights.svelte";
  import SuggestionComposer from "./SuggestionComposer.svelte";
  import HistoryModal from "./HistoryModal.svelte";

  let { view, title }: { view: EditorView | null; title: string } = $props();

  function clearSession() {
    reviewSessionStore.set(null);
    suggestingModeStore.set(false);
    reviewPanelOpenStore.set(false);
    historyOpenStore.set(false);
    resetComments();
    resetSuggestions();
  }

  $effect(() => {
    editorViewStore.set(view);
  });

  let collab = $derived(app.collab);
  let documentId = $derived(app.documentLink?.document_id ?? null);
  let role = $derived(app.documentLink?.role ?? null);
  let userId = $derived(app.account?.user_id ?? null);
  let username = $derived(app.account?.username ?? "");

  $effect(() => {
    if (!collab || !documentId || !role || !userId) {
      clearSession();
      return;
    }
    setLocalAwarenessUser(collab.provider.awareness, { user_id: userId, username });
    reviewSessionStore.set({ documentId, role, userId, doc: collab.doc, text: collab.text });
    suggestingModeStore.set(role === "suggester");
    return clearSession;
  });
</script>

{#if $reviewSessionStore}
  {#key $reviewSessionStore}
    <CommentHighlights />
    <SuggestionHighlights />
    <SuggestionComposer docId={$reviewSessionStore.documentId} />
    {#if $historyOpenStore}
      <HistoryModal {title} onClose={() => historyOpenStore.set(false)} />
    {/if}
  {/key}
{/if}
