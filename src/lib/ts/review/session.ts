import { writable } from "svelte/store";
import type { EditorView } from "@codemirror/view";
import type * as Y from "yjs";

export interface ReviewSession {
  documentId: string;
  role: string;
  userId: string;
  doc: Y.Doc;
  text: Y.Text;
}

export const reviewSessionStore = writable<ReviewSession | null>(null);
export const editorViewStore = writable<EditorView | null>(null);

export function formatTimestamp(value: string) {
  const normalized = value.includes("T") || value.endsWith("Z") ? value : value.replace(" ", "T") + "Z";
  const date = new Date(normalized);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}
