<script lang="ts">
  import type { EditorView } from "@codemirror/view";
  import { app } from "#lib/ts/state.svelte";
  import { editorViewStore, reviewSessionStore, type ReviewScope } from "#lib/ts/review/session";
  import { historyOpenStore, reviewPanelOpenStore, reviewRequest } from "#lib/ts/review/client";
  import { projectScopeFromRoom } from "#lib/ts/review/endpoints";
  import { setLocalAwarenessUser } from "#lib/ts/review/awareness";
  import { resetComments } from "#lib/ts/review/comments";
  import { resetSuggestions, suggestingModeStore } from "#lib/ts/review/suggestions";
  import CommentHighlights from "./CommentHighlights.svelte";
  import SuggestionHighlights from "./SuggestionHighlights.svelte";
  import SuggestionComposer from "./SuggestionComposer.svelte";
  import HistoryModal from "./HistoryModal.svelte";

  let { view, title }: { view: EditorView | null; title: string } = $props();

  let projectRole = $state<string | null>(null);

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
  let projectScope = $derived(collab && !documentId ? projectScopeFromRoom(collab.provider.roomname) : null);
  let scope = $derived<ReviewScope | null>(documentId ? { kind: "document", id: documentId, fileId: null } : projectScope);
  let role = $derived(documentId ? (app.documentLink?.role ?? null) : projectRole);
  let userId = $derived(app.account?.user_id ?? null);
  let username = $derived(app.account?.username ?? "");

  $effect(() => {
    const currentProject = projectScope;
    projectRole = null;
    if (!currentProject) return;
    let cancelled = false;
    reviewRequest<{ role?: string }>("GET", `/projects/${currentProject.id}/manifest`)
      .then((manifest) => {
        if (!cancelled) projectRole = manifest.role ?? null;
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    if (!collab || !scope || !role || !userId) {
      clearSession();
      return;
    }
    setLocalAwarenessUser(collab.provider.awareness, { user_id: userId, username });
    reviewSessionStore.set({ scope, role, userId, doc: collab.doc, text: collab.text });
    suggestingModeStore.set(role === "suggester");
    return clearSession;
  });
</script>

{#if $reviewSessionStore}
  {#key $reviewSessionStore}
    <CommentHighlights />
    <SuggestionHighlights />
    <SuggestionComposer scope={$reviewSessionStore.scope} />
    {#if $historyOpenStore}
      <HistoryModal {title} onClose={() => historyOpenStore.set(false)} />
    {/if}
  {/key}
{/if}
