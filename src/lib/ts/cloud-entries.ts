import * as api from "./api";
import {
  allSortOptions,
  defaultViewFilters,
  matchesQuery,
  parseTimestamp,
  sortByFilters,
  type ViewFilters,
} from "./view-filters";
import type {
  CloudDocument,
  CloudFile,
  CloudFolder,
  LinkedDocument,
  LinkedProject,
  ProjectSummary,
} from "./api";

export type CloudKind = "folder" | "project" | "document" | "file";
export type { ViewMode } from "./view-filters";
export type CloudFilters = ViewFilters;

export const defaultCloudFilters: CloudFilters = defaultViewFilters;

export const cloudSortOptions = allSortOptions.filter((option) => option.value !== "size");

export interface CloudAction {
  label: string;
  icon: string;
  color?: string;
  danger?: boolean;
  run: () => void;
}

export interface CloudEntry {
  key: string;
  kind: CloudKind;
  id: string;
  name: string;
  detail: string;
  modified: string | null;
  localPath: string | null;
  thumbnailSvg: string | null;
  syncState: "synced" | "pending" | null;
  color: string | null;
  setColor: ((color: string | null) => void) | null;
  open: () => void;
  actions: CloudAction[];
}

export interface CloudHandlers {
  openFolder: (folder: CloudFolder) => void;
  openLocal: (path: string) => void;
  downloadDocument: (documentId: string, title: string) => void;
  removeDownload: (path: string) => void;
  deleteDocument: (documentId: string) => void;
  cloneProject: (projectId: string, name: string) => void;
  deleteProject: (projectId: string) => void;
  renameFolder: (folder: CloudFolder) => void;
  deleteFolder: (folder: CloudFolder) => void;
  setFolderColor: (folder: CloudFolder, color: string | null) => void;
  downloadFile: (fileId: string, name: string) => void;
  openFile: (fileId: string, name: string) => void;
  renameFile: (file: CloudFile) => void;
  deleteFile: (fileId: string) => void;
}

export function formatShortDate(value: string | null) {
  const time = parseTimestamp(value);
  if (!time) return "";
  return new Date(time).toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

function capitalize(value: string) {
  return value.charAt(0).toUpperCase() + value.slice(1);
}

export function folderEntry(folder: CloudFolder, handlers: CloudHandlers): CloudEntry {
  return {
    key: `folder:${folder.id}`,
    kind: "folder",
    id: folder.id,
    name: folder.name,
    detail: "Folder",
    modified: null,
    localPath: null,
    thumbnailSvg: null,
    syncState: null,
    color: folder.color ?? null,
    setColor: (color) => handlers.setFolderColor(folder, color),
    open: () => handlers.openFolder(folder),
    actions: [
      { label: "Open", icon: "ph:folder-open", run: () => handlers.openFolder(folder) },
      { label: "Rename", icon: "ph:pencil-simple", run: () => handlers.renameFolder(folder) },
      { label: "Delete", icon: "ph:trash", danger: true, run: () => handlers.deleteFolder(folder) },
    ],
  };
}

export function projectEntry(
  project: ProjectSummary,
  linked: LinkedProject | undefined,
  handlers: CloudHandlers,
): CloudEntry {
  const open = linked
    ? () => handlers.openLocal(linked.path)
    : () => handlers.cloneProject(project.id, project.name);
  const actions: CloudAction[] = [
    linked
      ? { label: "Open", icon: "ph:arrow-square-out", run: open }
      : { label: "Download to this device", icon: "ph:download-simple", run: open },
  ];
  if (project.role === "owner") {
    actions.push({ label: "Delete", icon: "ph:trash", danger: true, run: () => handlers.deleteProject(project.id) });
  }

  return {
    key: `project:${project.id}`,
    kind: "project",
    id: project.id,
    name: project.name,
    detail: project.role === "owner" ? "Project" : capitalize(project.role),
    modified: project.updated_at,
    localPath: linked?.path ?? null,
    thumbnailSvg: null,
    syncState: linked ? (linked.sync_state ?? "synced") : null,
    color: null,
    setColor: null,
    open,
    actions,
  };
}

export function documentEntry(
  document: CloudDocument,
  linked: LinkedDocument | undefined,
  handlers: CloudHandlers,
): CloudEntry {
  const open = linked
    ? () => handlers.openLocal(linked.path)
    : () => handlers.downloadDocument(document.id, document.title);
  const actions: CloudAction[] = [
    linked
      ? { label: "Open", icon: "ph:arrow-square-out", run: open }
      : { label: "Download to this device", icon: "ph:download-simple", run: open },
  ];
  if (linked) {
    actions.push({ label: "Remove from this device", icon: "ph:minus-circle", run: () => handlers.removeDownload(linked.path) });
  }
  if (document.role === "owner") {
    actions.push({ label: "Delete", icon: "ph:trash", danger: true, run: () => handlers.deleteDocument(document.id) });
  }

  return {
    key: `document:${document.id}`,
    kind: "document",
    id: document.id,
    name: document.title,
    detail: document.role === "owner" ? "Document" : capitalize(document.role),
    modified: document.updated_at,
    localPath: linked?.path ?? null,
    thumbnailSvg: document.thumbnail_svg ?? null,
    syncState: linked ? (linked.sync_state ?? "synced") : null,
    color: null,
    setColor: null,
    open,
    actions,
  };
}

export function fileEntry(file: CloudFile, handlers: CloudHandlers): CloudEntry {
  const addToAssets = () => handlers.downloadFile(file.id, file.name);
  return {
    key: `file:${file.id}`,
    kind: "file",
    id: file.id,
    name: file.name,
    detail: fileKindLabel(file.name),
    modified: file.created_at,
    localPath: null,
    thumbnailSvg: null,
    syncState: null,
    color: null,
    setColor: null,
    open: () => handlers.openFile(file.id, file.name),
    actions: [
      { label: "Open", icon: "ph:arrow-square-out", run: () => handlers.openFile(file.id, file.name) },
      { label: "Add to assets", icon: "ph:download-simple", run: addToAssets },
      { label: "Rename", icon: "ph:pencil-simple", run: () => handlers.renameFile(file) },
      { label: "Delete", icon: "ph:trash", danger: true, run: () => handlers.deleteFile(file.id) },
    ],
  };
}

export function isFontName(name: string) {
  return /\.(ttf|otf|ttc|otc|woff2?)$/i.test(name);
}

function fileKindLabel(name: string) {
  if (api.isImagePath(name)) return "Image";
  if (isFontName(name)) return "Font";
  if (/\.pdf$/i.test(name)) return "PDF";
  return "File";
}

export function fileIcon(name: string) {
  if (api.isImagePath(name)) return "ph:image";
  if (isFontName(name)) return "ph:text-aa";
  if (/\.pdf$/i.test(name)) return "ph:file-pdf";
  return "ph:file";
}

export function matchesSearch(entry: CloudEntry, query: string) {
  return matchesQuery(entry.name, query);
}

export function sortEntries(entries: CloudEntry[], filters: CloudFilters) {
  return sortByFilters(entries, filters, (entry) => entry);
}
