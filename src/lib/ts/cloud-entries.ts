import * as api from "./api";
import type {
  CloudDocument,
  CloudFile,
  CloudFolder,
  LinkedDocument,
  LinkedProject,
  ProjectSummary,
} from "./api";

export type CloudKind = "folder" | "project" | "document" | "file";
export type SortField = "name" | "modified";
export type SortDirection = "asc" | "desc";
export type ViewMode = "grid" | "list";

export interface CloudFilters {
  sortField: SortField;
  sortDirection: SortDirection;
}

export const defaultCloudFilters: CloudFilters = {
  sortField: "name",
  sortDirection: "asc",
};

export const sortOptions: { value: SortField; label: string }[] = [
  { value: "name", label: "Name" },
  { value: "modified", label: "Last modified" },
];

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

function parseTimestamp(value: string | null) {
  if (!value) return 0;
  const normalized =
    value.includes("T") || value.endsWith("Z") ? value : value.replace(" ", "T") + "Z";
  const time = Date.parse(normalized);
  return Number.isNaN(time) ? 0 : time;
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
    syncState: null,
    color: folder.color ?? null,
    setColor: (color) => handlers.setFolderColor(folder, color),
    open: () => handlers.openFolder(folder),
    actions: [
      { label: "Open", icon: "mdi:folder-open-outline", color: "text-green-500", run: () => handlers.openFolder(folder) },
      { label: "Rename", icon: "mdi:pencil-outline", color: "text-yellow-500", run: () => handlers.renameFolder(folder) },
      { label: "Delete", icon: "mdi:trash-can-outline", danger: true, run: () => handlers.deleteFolder(folder) },
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
      ? { label: "Open", icon: "mdi:open-in-new", color: "text-green-500", run: open }
      : { label: "Download to this device", icon: "mdi:download", color: "text-teal-500", run: open },
  ];
  if (project.role === "owner") {
    actions.push({ label: "Delete", icon: "mdi:trash-can-outline", danger: true, run: () => handlers.deleteProject(project.id) });
  }

  return {
    key: `project:${project.id}`,
    kind: "project",
    id: project.id,
    name: project.name,
    detail: project.role === "owner" ? "Project" : capitalize(project.role),
    modified: project.updated_at,
    localPath: linked?.path ?? null,
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
      ? { label: "Open", icon: "mdi:open-in-new", color: "text-green-500", run: open }
      : { label: "Download to this device", icon: "mdi:download", color: "text-teal-500", run: open },
  ];
  if (linked) {
    actions.push({ label: "Remove from this device", icon: "mdi:laptop-off", color: "text-yellow-500", run: () => handlers.removeDownload(linked.path) });
  }
  if (document.role === "owner") {
    actions.push({ label: "Delete", icon: "mdi:trash-can-outline", danger: true, run: () => handlers.deleteDocument(document.id) });
  }

  return {
    key: `document:${document.id}`,
    kind: "document",
    id: document.id,
    name: document.title,
    detail: document.role === "owner" ? "Document" : capitalize(document.role),
    modified: document.updated_at,
    localPath: linked?.path ?? null,
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
    syncState: null,
    color: null,
    setColor: null,
    open: () => handlers.openFile(file.id, file.name),
    actions: [
      { label: "Open", icon: "mdi:open-in-new", color: "text-green-500", run: () => handlers.openFile(file.id, file.name) },
      { label: "Add to assets", icon: "mdi:download", color: "text-teal-500", run: addToAssets },
      { label: "Rename", icon: "mdi:pencil-outline", color: "text-yellow-500", run: () => handlers.renameFile(file) },
      { label: "Delete", icon: "mdi:trash-can-outline", danger: true, run: () => handlers.deleteFile(file.id) },
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
  if (api.isImagePath(name)) return "mdi:image-outline";
  if (isFontName(name)) return "mdi:format-font";
  if (/\.pdf$/i.test(name)) return "mdi:file-pdf-box";
  return "mdi:file-outline";
}

export function matchesSearch(entry: CloudEntry, query: string) {
  const normalized = query.trim().toLowerCase();
  return !normalized || entry.name.toLowerCase().includes(normalized);
}

export function sortEntries(entries: CloudEntry[], filters: CloudFilters) {
  const direction = filters.sortDirection === "asc" ? 1 : -1;
  return [...entries].sort((first, second) => {
    let difference = 0;
    if (filters.sortField === "modified") {
      difference = parseTimestamp(first.modified) - parseTimestamp(second.modified);
    }
    if (difference === 0) {
      difference = first.name.localeCompare(second.name, undefined, {
        numeric: true,
        sensitivity: "base",
      });
    }
    return difference * direction;
  });
}
