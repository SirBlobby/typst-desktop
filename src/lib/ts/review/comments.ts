import { writable } from "svelte/store";
import type { EditorView } from "@codemirror/view";
import type * as Y from "yjs";
import type { PersonSummary } from "./people";
import { anchorRange, type AnchoredRange } from "./anchors";
import { reviewPanelOpenStore, reviewRequest } from "./client";

export interface CommentView {
  id: string;
  document_id: string;
  parent_id: string | null;
  author: PersonSummary | null;
  user_id: string;
  content: string;
  resolved: boolean;
  anchor_start: string | null;
  anchor_end: string | null;
  quoted_text: string | null;
  created_at: string;
  updated_at: string | null;
}

export interface CommentThread {
  root: CommentView;
  replies: CommentView[];
}

export interface PendingComment extends AnchoredRange {
  quoted_text: string;
}

export const commentsStore = writable<CommentView[]>([]);
export const activeThreadStore = writable<string | null>(null);
export const pendingCommentStore = writable<PendingComment | null>(null);

export async function loadComments(documentId: string) {
  commentsStore.set(await reviewRequest<CommentView[]>("GET", `/documents/${documentId}/comments`));
}

export async function createComment(
  documentId: string,
  content: string,
  options: { parentId?: string; pending?: PendingComment | null } = {},
): Promise<CommentView> {
  const created = await reviewRequest<CommentView>("POST", `/documents/${documentId}/comments`, {
    content,
    parent_id: options.parentId ?? null,
    anchor_start: options.pending?.anchor_start ?? null,
    anchor_end: options.pending?.anchor_end ?? null,
    quoted_text: options.pending?.quoted_text ?? null,
  });
  await loadComments(documentId);
  return created;
}

export async function updateComment(documentId: string, commentId: string, changes: { content?: string; resolved?: boolean }) {
  await reviewRequest("PATCH", `/comments/${commentId}`, changes);
  await loadComments(documentId);
}

export async function deleteComment(documentId: string, commentId: string) {
  await reviewRequest("DELETE", `/comments/${commentId}`);
  await loadComments(documentId);
}

export function buildThreads(comments: CommentView[]): CommentThread[] {
  const threads = new Map<string, CommentThread>();
  for (const comment of comments) {
    if (!comment.parent_id) threads.set(comment.id, { root: comment, replies: [] });
  }
  for (const comment of comments) {
    if (comment.parent_id) threads.get(comment.parent_id)?.replies.push(comment);
  }
  return [...threads.values()];
}

export function startCommentOnSelection(view: EditorView | null, text: Y.Text | null) {
  if (view && text) {
    const selection = view.state.selection.main;
    if (selection.to > selection.from) {
      pendingCommentStore.set({
        ...anchorRange(text, selection.from, selection.to),
        quoted_text: view.state.doc.sliceString(selection.from, selection.to),
      });
    }
  }
  reviewPanelOpenStore.set(true);
}

export function resetComments() {
  commentsStore.set([]);
  activeThreadStore.set(null);
  pendingCommentStore.set(null);
}
