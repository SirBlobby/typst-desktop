import type { PersonSummary } from "./people";
import { reviewRequest } from "./client";

export type VersionKind = "manual" | "auto" | "restore";

export interface DocumentVersion {
  id: string;
  document_id: string;
  author: PersonSummary | null;
  editors: PersonSummary[];
  content: string;
  created_at: string;
  updated_at: string | null;
  kind: VersionKind;
  label: string | null;
}

export const versionKindLabels: Record<VersionKind, string> = {
  manual: "Saved",
  auto: "Auto saved",
  restore: "Restored"
};

export function fetchVersions(documentId: string) {
  return reviewRequest<DocumentVersion[]>("GET", `/documents/${documentId}/versions`);
}

export function saveVersion(documentId: string, label?: string) {
  return reviewRequest<{ created: boolean; version: DocumentVersion | null }>("POST", `/documents/${documentId}/versions`, { label: label ?? null });
}

export async function renameVersion(documentId: string, versionId: string, label: string) {
  await reviewRequest("PATCH", `/documents/${documentId}/versions/${versionId}`, { label });
}

export async function restoreVersion(documentId: string, versionId: string) {
  await reviewRequest("POST", `/documents/${documentId}/versions/${versionId}/restore`);
}

function parseTimestamp(value: string) {
  const normalized = value.includes("T") || value.endsWith("Z") ? value : value.replace(" ", "T") + "Z";
  return new Date(normalized);
}

function clockTime(value: string) {
  return parseTimestamp(value).toLocaleTimeString(undefined, { hour: "numeric", minute: "2-digit" });
}

export function versionTime(version: DocumentVersion) {
  const startTime = clockTime(version.created_at);
  if (!version.updated_at) return startTime;
  const endTime = clockTime(version.updated_at);
  return endTime === startTime ? startTime : `${startTime} - ${endTime}`;
}

export function versionDay(version: DocumentVersion) {
  const date = parseTimestamp(version.updated_at ?? version.created_at);
  const today = new Date();
  const yesterday = new Date();
  yesterday.setDate(today.getDate() - 1);
  if (date.toDateString() === today.toDateString()) return "Today";
  if (date.toDateString() === yesterday.toDateString()) return "Yesterday";
  return date.toLocaleDateString(undefined, { weekday: "long", month: "long", day: "numeric", year: "numeric" });
}

export function groupByDay(versions: DocumentVersion[]) {
  const groups: { day: string; versions: DocumentVersion[] }[] = [];
  for (const version of versions) {
    const day = versionDay(version);
    const last = groups[groups.length - 1];
    if (last && last.day === day) last.versions.push(version);
    else groups.push({ day, versions: [version] });
  }
  return groups;
}
