import { writable } from "svelte/store";
import type * as Y from "yjs";
import type { PersonSummary } from "./people";
import { resolveRange, type AnchoredRange } from "./anchors";
import { reviewRequest } from "./client";
import { createPath, inScope, itemPath, listPath } from "./endpoints";
import type { ReviewScope } from "./session";

export type SuggestionStatus = "pending" | "accepted" | "rejected" | "withdrawn";

export interface SuggestionView {
  id: string;
  document_id?: string;
  file_id?: string;
  user_id: string;
  author: PersonSummary | null;
  anchor_start: string;
  anchor_end: string;
  original_text: string;
  replacement_text: string;
  note: string | null;
  status: SuggestionStatus;
  resolved_by: PersonSummary | null;
  resolved_at: string | null;
  created_at: string;
}

export interface SuggestionDraft extends AnchoredRange {
  from: number;
  to: number;
  original_text: string;
  replacement_text: string;
}

export const suggestionsStore = writable<SuggestionView[]>([]);
export const suggestionDraftStore = writable<SuggestionDraft | null>(null);
export const suggestingModeStore = writable(false);
export const activeSuggestionStore = writable<string | null>(null);

export async function loadSuggestions(scope: ReviewScope) {
  const suggestions = await reviewRequest<SuggestionView[]>("GET", listPath(scope, "suggestions"));
  suggestionsStore.set(inScope(scope, suggestions));
}

export async function submitSuggestion(scope: ReviewScope, draft: SuggestionDraft, note: string) {
  await reviewRequest("POST", createPath(scope, "suggestions"), {
    anchor_start: draft.anchor_start,
    anchor_end: draft.anchor_end,
    original_text: draft.original_text,
    replacement_text: draft.replacement_text,
    note: note || null
  });
  await loadSuggestions(scope);
}

async function setStatus(scope: ReviewScope, suggestionId: string, status: SuggestionStatus) {
  await reviewRequest("PATCH", itemPath(scope, "suggestions", suggestionId), { status });
  await loadSuggestions(scope);
}

export function currentSuggestionRange(doc: Y.Doc, suggestion: SuggestionView) {
  return resolveRange(doc, suggestion);
}

export async function acceptSuggestion(scope: ReviewScope, doc: Y.Doc, text: Y.Text, suggestion: SuggestionView) {
  const range = currentSuggestionRange(doc, suggestion);
  if (!range) throw new Error("The text this suggestion refers to no longer exists.");
  doc.transact(() => {
    if (range.to > range.from) text.delete(range.from, range.to - range.from);
    if (suggestion.replacement_text) text.insert(range.from, suggestion.replacement_text);
  });
  await setStatus(scope, suggestion.id, "accepted");
}

export function rejectSuggestion(scope: ReviewScope, suggestion: SuggestionView) {
  return setStatus(scope, suggestion.id, "rejected");
}

export function withdrawSuggestion(scope: ReviewScope, suggestion: SuggestionView) {
  return setStatus(scope, suggestion.id, "withdrawn");
}

export function textChangedSince(doc: Y.Doc, text: Y.Text, suggestion: SuggestionView) {
  const range = currentSuggestionRange(doc, suggestion);
  if (!range) return true;
  return text.toString().slice(range.from, range.to) !== suggestion.original_text;
}

export function resetSuggestions() {
  suggestionsStore.set([]);
  suggestionDraftStore.set(null);
  activeSuggestionStore.set(null);
}
