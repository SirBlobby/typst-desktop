<script lang="ts">
  import { onMount, untrack } from "svelte";
  import Icon from "@iconify/svelte";
  import ConfirmModal from "$lib/components/ConfirmModal.svelte";
  import CommentThreadCard from "./CommentThreadCard.svelte";
  import CommentComposer from "./CommentComposer.svelte";
  import SuggestionsList from "./SuggestionsList.svelte";
  import { activeSuggestionStore, loadSuggestions, suggestionsStore } from "$lib/ts/review/suggestions";
  import { get } from "svelte/store";
  import { editorViewStore, reviewSessionStore } from "$lib/ts/review/session";
  import { resolveRange } from "$lib/ts/review/anchors";
  import {
    activeThreadStore,
    buildThreads,
    commentsStore,
    createComment,
    deleteComment,
    loadComments,
    pendingCommentStore,
    type CommentThread,
    type CommentView
  } from "$lib/ts/review/comments";

  type PanelTab = "open" | "resolved" | "suggestions";

  let { docId, canComment, canReview, isOwner, onClose }: {
    docId: string;
    canComment: boolean;
    canReview: boolean;
    isOwner: boolean;
    onClose: () => void;
  } = $props();

  const { doc } = get(reviewSessionStore)!;

  const REFRESH_INTERVAL_MS = 15000;

  let filter = $state<PanelTab>("open");
  let loading = $state(true);
  let loadError = $state("");
  let commentToDelete = $state<CommentView | null>(null);

  let threads = $derived(buildThreads($commentsStore));
  let openCount = $derived(threads.filter((thread) => !thread.root.resolved).length);
  let pendingSuggestionCount = $derived($suggestionsStore.filter((suggestion) => suggestion.status === "pending").length);
  let tabs = $derived<{ value: PanelTab; label: string; count: number }[]>([
    { value: "open", label: "Open", count: openCount },
    { value: "resolved", label: "Resolved", count: 0 },
    { value: "suggestions", label: "Suggestions", count: pendingSuggestionCount }
  ]);
  let visibleThreads = $derived(threads.filter((thread) => (filter === "open" ? !thread.root.resolved : thread.root.resolved)));

  async function refresh() {
    try {
      await Promise.all([loadComments(docId), loadSuggestions(docId)]);
      loadError = "";
    } catch (err) {
      loadError = (err as Error).message;
    }
    loading = false;
  }

  function jumpToThread(thread: CommentThread) {
    const view = $editorViewStore;
    if (!view || !doc) return;
    const range = resolveRange(doc, thread.root);
    if (!range) return;
    view.dispatch({ selection: { anchor: range.from, head: range.to }, scrollIntoView: true });
  }

  function toggleThread(thread: CommentThread) {
    if ($activeThreadStore === thread.root.id) {
      activeThreadStore.set(null);
      return;
    }
    activeThreadStore.set(thread.root.id);
    jumpToThread(thread);
  }

  async function startThread(content: string) {
    const created = await createComment(docId, content, { pending: $pendingCommentStore });
    pendingCommentStore.set(null);
    filter = "open";
    activeThreadStore.set(created.id);
  }

  async function confirmDelete() {
    const comment = commentToDelete;
    commentToDelete = null;
    if (!comment) return;
    try {
      await deleteComment(docId, comment.id);
    } catch (err) {
      alert((err as Error).message);
    }
  }

  $effect(() => {
    const activeId = $activeThreadStore;
    if (!activeId) return;
    untrack(() => {
      const activeThread = threads.find((thread) => thread.root.id === activeId);
      if (activeThread) filter = activeThread.root.resolved ? "resolved" : "open";
      activeSuggestionStore.set(null);
    });
  });

  $effect(() => {
    if (!$activeSuggestionStore) return;
    untrack(() => {
      filter = "suggestions";
      activeThreadStore.set(null);
    });
  });

  onMount(() => {
    refresh();
    const timer = setInterval(refresh, REFRESH_INTERVAL_MS);
    return () => clearInterval(timer);
  });
</script>

<div class="flex h-full w-80 shrink-0 flex-col border-l border-[var(--color-line)] bg-[var(--color-surface)] text-[var(--color-ink)]">
  <div class="bar justify-between pl-4 pr-1.5">
    <div class="flex items-center gap-2">
      <Icon icon="ph:chats" class="text-sm text-[var(--color-ink-subtle)]" />
      <h2 class="text-xs font-semibold">Comments and suggestions</h2>
    </div>
    <button onclick={onClose} class="icon-btn" title="Close comments" aria-label="Close comments">
      <Icon icon="ph:x" class="text-sm" />
    </button>
  </div>

  <div class="m-3 mb-0 flex gap-0.5 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-sunken)] p-0.5">
    {#each tabs as tab (tab.value)}
      <button
        type="button"
        onclick={() => (filter = tab.value)}
        class="flex h-6 flex-1 items-center justify-center gap-1 rounded-md text-xs font-medium transition {filter === tab.value ? 'bg-[var(--color-surface-raised)] text-[var(--color-ink)] shadow-sm' : 'text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
      >
        {tab.label}
        {#if tab.count > 0}
          <span class="rounded-full bg-[var(--color-accent-soft)] px-1.5 text-[10px] font-semibold text-[var(--color-accent)]">{tab.count}</span>
        {/if}
      </button>
    {/each}
  </div>

  <div class="scroll-thin flex-1 space-y-2.5 overflow-y-auto p-3">
    {#if loading}
      <div class="flex h-full items-center justify-center"><Icon icon="ph:circle-notch" class="animate-spin text-2xl" /></div>
    {:else if filter === 'suggestions'}
      <SuggestionsList {docId} {canReview} />
    {:else if loadError}
      <p class="rounded-lg border border-[var(--color-danger)]/20 bg-[var(--color-danger)]/8 p-3 text-center text-xs text-[var(--color-danger)]">{loadError}</p>
    {:else if visibleThreads.length === 0}
      <div class="empty-state h-full">
        <Icon icon={filter === 'open' ? 'ph:chat-circle' : 'ph:checks'} class="text-3xl" />
        <p class="text-[13px] text-[var(--color-ink-muted)]">{filter === 'open' ? 'No open comments' : 'No resolved comments'}</p>
        {#if filter === 'open' && canComment}
          <p class="max-w-[16rem] text-xs">Select text in the editor and press Ctrl+Alt+M to comment on it.</p>
        {/if}
      </div>
    {:else}
      {#each visibleThreads as thread (thread.root.id)}
        <CommentThreadCard
          {thread}
          {docId}
          {canComment}
          {isOwner}
          expanded={$activeThreadStore === thread.root.id}
          onToggle={() => toggleThread(thread)}
          onRequestDelete={(comment) => (commentToDelete = comment)}
        />
      {/each}
    {/if}
  </div>

  {#if canComment && filter !== 'suggestions'}
    <div class="border-t border-[var(--color-line)] p-3">
      {#if $pendingCommentStore}
        <div class="mb-2 flex items-start gap-2 rounded-md bg-amber-500/10 px-2.5 py-2">
          <p class="min-w-0 flex-1 truncate border-l-2 border-amber-400 pl-2 text-xs italic text-[var(--color-ink-muted)]" title={$pendingCommentStore.quoted_text}>
            {$pendingCommentStore.quoted_text}
          </p>
          <button type="button" onclick={() => pendingCommentStore.set(null)} title="Comment on the whole document instead" aria-label="Remove selected text" class="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]">
            <Icon icon="ph:x" class="text-sm" />
          </button>
        </div>
      {/if}
      <CommentComposer
        placeholder={$pendingCommentStore ? 'Comment on the selected text...' : 'Comment on the document...'}
        submitLabel="Comment"
        onSubmit={startThread}
      />
    </div>
  {/if}
</div>

{#if commentToDelete}
  <ConfirmModal
    title="Delete comment"
    message={commentToDelete.parent_id ? 'This reply will be permanently deleted.' : 'This comment and all of its replies will be permanently deleted.'}
    confirmLabel="Delete"
    onconfirm={confirmDelete}
    onclose={() => (commentToDelete = null)}
  />
{/if}
