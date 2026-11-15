import type { ReviewScope } from "./session";

export type ReviewKind = "comments" | "suggestions";

export interface ScopedItem {
  file_id?: string | null;
}

const PROJECT_ROOM_PREFIX = "project:";

export function listPath(scope: ReviewScope, kind: ReviewKind) {
  const base = scope.kind === "document" ? "documents" : "projects";
  return `/${base}/${scope.id}/${kind}`;
}

export function createPath(scope: ReviewScope, kind: ReviewKind) {
  if (scope.kind === "document") return `/documents/${scope.id}/${kind}`;
  return `/projects/${scope.id}/files/${scope.fileId}/${kind}`;
}

export function itemPath(scope: ReviewScope, kind: ReviewKind, itemId: string) {
  const prefix = scope.kind === "document" ? "" : "project-";
  return `/${prefix}${kind}/${itemId}`;
}

export function inScope<T extends ScopedItem>(scope: ReviewScope, items: T[]) {
  if (scope.kind === "document") return items;
  return items.filter((item) => item.file_id === scope.fileId);
}

export function projectScopeFromRoom(roomName: string): ReviewScope | null {
  if (!roomName.startsWith(PROJECT_ROOM_PREFIX)) return null;
  const [projectId, fileId] = roomName.slice(PROJECT_ROOM_PREFIX.length).split(":");
  if (!projectId || !fileId) return null;
  return { kind: "project", id: projectId, fileId };
}
