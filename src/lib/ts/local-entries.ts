import * as api from "./api";
import type { BrowseEntry } from "./api";

const kindIcons = {
  project: "ph:folder-star",
  folder: "ph:folder",
  document: "ph:file-text",
  file: "ph:file",
};

export function entryIcon(entry: BrowseEntry): string {
  if (entry.kind === "file" && api.isPdfPath(entry.path)) return "ph:file-pdf";
  if (entry.kind === "file" && api.isImagePath(entry.path)) return "ph:image";
  return kindIcons[entry.kind];
}

export function entryTypeLabel(entry: BrowseEntry): string {
  if (entry.kind === "project") return "Project";
  if (entry.kind === "folder") return "Folder";
  if (entry.kind === "document") return "Document";
  if (api.isPdfPath(entry.path)) return "PDF";
  if (api.isImagePath(entry.path)) return "Image";
  return "File";
}

export function isContainer(entry: BrowseEntry): boolean {
  return entry.kind === "folder" || entry.kind === "project";
}

export function relativeTime(value: string | null): string {
  if (!value) return "";
  const then = new Date(value).getTime();
  if (Number.isNaN(then)) return "";

  const minutes = Math.round((Date.now() - then) / 60000);
  if (minutes < 1) return "just now";
  if (minutes < 60) return `${minutes}m ago`;
  if (minutes < 1440) return `${Math.round(minutes / 60)}h ago`;
  return `${Math.round(minutes / 1440)}d ago`;
}

export function syncLabel(entry: BrowseEntry): string {
  if (entry.sync_state === "pending") return "Local changes not yet synced";
  if (entry.sync_state === "synced") {
    const when = relativeTime(entry.last_synced_at);
    return when ? `Synced ${when}` : "Synced";
  }
  return "Linked to the cloud";
}

export function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export function formatDate(value: string | null): string {
  if (!value) return "";
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? "" : date.toLocaleDateString();
}

export function sizeLabel(entry: BrowseEntry): string {
  return isContainer(entry) ? `${entry.child_count} items` : formatSize(entry.size);
}
