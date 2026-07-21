import { invoke } from "@tauri-apps/api/core";

export interface Settings {
  workspace_root: string;
  server_url: string;
  device_token: string | null;
  account_email: string | null;
  account_username: string | null;
  autosave_seconds: number;
  sync_seconds: number;
}

export interface FileEntry {
  path: string;
  name: string;
  is_dir: boolean;
  is_text: boolean;
  size: number;
}

export interface FilePayload {
  path: string;
  is_text: boolean;
  content: string;
}

export interface Diagnostic {
  message: string;
  severity: string;
  line: number | null;
  column: number | null;
}

export interface DocumentStats {
  pages: number;
  words: number;
  characters: number;
}

export interface CompileResult {
  pages: string[];
  stats: DocumentStats;
  diagnostics: Diagnostic[];
}

export interface CompileFailure {
  diagnostics: Diagnostic[];
}

export interface Account {
  user_id: string;
  username: string;
  email: string;
}

export interface ProjectSummary {
  id: string;
  name: string;
  entrypoint: string;
  folder_id: string | null;
  role: string;
  updated_at: string;
}

export interface Conflict {
  path: string;
  local_text: string;
  remote_text: string;
  merged_text: string;
  server_hash: string;
  auto_merged: boolean;
  binary: boolean;
}

export interface SyncReport {
  pushed: string[];
  pulled: string[];
  deleted_local: string[];
  deleted_remote: string[];
  merged: string[];
  conflicts: Conflict[];
}

export interface Resolution {
  path: string;
  content: string;
  server_hash: string;
}

export type EntryKind = "folder" | "project" | "document" | "file";

export interface BrowseEntry {
  name: string;
  path: string;
  kind: EntryKind;
  size: number;
  modified: string | null;
  cloud_project_id: string | null;
  last_synced_at: string | null;
  child_count: number;
  cloud_linked: boolean;
  sync_state: "synced" | "pending" | null;
}

export interface TargetInfo {
  path: string;
  entrypoint: string;
  standalone: boolean;
  is_project: boolean;
  cloud_project_id: string | null;
  files: FileEntry[];
}

export const browseWorkspace = (path: string) =>
  invoke<BrowseEntry[]>("browse_workspace", { path });

export const createFolderEntry = (parent: string, name: string) =>
  invoke<string>("create_folder_entry", { parent, name });

export const createDocumentEntry = (parent: string, name: string) =>
  invoke<string>("create_document_entry", { parent, name });

export const createProjectEntry = (parent: string, name: string) =>
  invoke<string>("create_project_entry", { parent, name });

export const renameEntry = (path: string, newName: string) =>
  invoke<string>("rename_entry", { path, newName });

export const deleteEntry = (path: string) =>
  invoke<void>("delete_entry", { path });

export const moveEntry = (path: string, destination: string) =>
  invoke<string>("move_entry", { path, destination });

export const duplicateEntry = (path: string) =>
  invoke<string>("duplicate_entry", { path });

export const absolutePath = (path: string) =>
  invoke<string>("absolute_path", { path });

export const uploadEntry = (
  parent: string,
  name: string,
  base64Content: string,
) => invoke<string>("upload_entry", { parent, name, base64Content });

export const targetInfo = (path: string) =>
  invoke<TargetInfo>("target_info", { path });

export const readTargetFile = (path: string, file: string) =>
  invoke<FilePayload>("read_target_file", { path, file });

export const writeTargetFile = (path: string, file: string, content: string) =>
  invoke<void>("write_target_file", { path, file, content });

export const setTargetEntrypoint = (path: string, entrypoint: string) =>
  invoke<void>("set_target_entrypoint", { path, entrypoint });

export const compileTarget = (
  path: string,
  entrypoint?: string,
  overrides?: Record<string, string>,
) => invoke<CompileResult>("compile_target", { path, entrypoint, overrides });

export const exportTarget = (
  path: string,
  format: string,
  destination: string,
) => invoke<string>("export_target", { path, format, destination });

export const renderTargetPng = (path: string) =>
  invoke<number[]>("render_target_png", { path });

export interface Asset {
  name: string;
  kind: "font" | "image" | "file";
  size: number;
  font_families: string[];
}

export const listAssets = () => invoke<Asset[]>("list_assets");

export interface Resource {
  name: string;
  reference: string;
  path: string;
  scope: "shared" | "project";
  kind: "font" | "image" | "file";
  size: number;
  font_families: string[];
}

export const listResources = (path: string) =>
  invoke<Resource[]>("list_resources", { path });

export interface Thumbnail {
  kind: "svg" | "image";
  data: string;
}

export const thumbnail = (path: string) =>
  invoke<Thumbnail>("thumbnail", { path });

export const clearThumbnails = () => invoke<void>("clear_thumbnails");

export interface ImageData {
  name: string;
  data: string;
  size: number;
  width: number | null;
  height: number | null;
}

export const readImage = (path: string) =>
  invoke<ImageData>("read_image", { path });

export const IMAGE_EXTENSIONS = ["png", "jpg", "jpeg", "gif", "svg", "webp"];

export function isImagePath(path: string): boolean {
  const extension = path.split(".").pop()?.toLowerCase() ?? "";
  return IMAGE_EXTENSIONS.includes(extension);
}

export const listFontFamilies = (path?: string) =>
  invoke<string[]>("list_font_families", { path: path ?? null });

export const importAssets = (sources: string[]) =>
  invoke<string[]>("import_assets", { sources });

export const deleteAsset = (name: string) =>
  invoke<void>("delete_asset", { name });

export const importIntoTarget = (path: string, sources: string[]) =>
  invoke<string[]>("import_into_target", { path, sources });

export const importIntoFolder = (parent: string, sources: string[]) =>
  invoke<string[]>("import_into_folder", { parent, sources });

export interface AppInfo {
  version: string;
  typst_version: string;
  authors: string;
  license: string;
  tauri_version: string;
}

export const appInfo = () => invoke<AppInfo>("app_info");

export const getSettings = () => invoke<Settings>("get_settings");

export const updateSettings = (changes: {
  workspaceRoot?: string;
  serverUrl?: string;
  autosaveSeconds?: number;
  syncSeconds?: number;
}) => invoke<Settings>("update_settings", changes);

export interface CompatibilityStatus {
  compatible: boolean;
  server_version: string;
  desktop_version: string;
  min_server_version: string;
  min_desktop_version: string;
  message: string | null;
}

export const cloudCheckCompatibility = (serverUrl: string) =>
  invoke<CompatibilityStatus>("cloud_check_compatibility", { serverUrl });

export const cloudLogin = (
  serverUrl: string,
  email: string,
  password: string,
) => invoke<Account>("cloud_login", { serverUrl, email, password });

export const cloudLogout = () => invoke<void>("cloud_logout");

export const cloudAccount = () => invoke<Account | null>("cloud_account");

export const cloudListProjects = () =>
  invoke<ProjectSummary[]>("cloud_list_projects");

export interface CloudFolder {
  id: string;
  name: string;
  parent_id: string | null;
}

export interface CloudDocument {
  id: string;
  title: string;
  folder_id: string | null;
  role: string;
  updated_at: string;
}

export interface SharedItems {
  documents: CloudDocument[];
  projects: ProjectSummary[];
}

export interface DocumentLink {
  document_id: string;
  base_hash: string;
  role: string;
  base_content: string;
}

export const cloudListFolders = () =>
  invoke<CloudFolder[]>("cloud_list_folders");

export const cloudCreateFolder = (name: string, parentId?: string | null) =>
  invoke<CloudFolder>("cloud_create_folder", {
    name,
    parentId: parentId ?? null,
  });

export const cloudRenameFolder = (folderId: string, name: string) =>
  invoke<CloudFolder>("cloud_rename_folder", { folderId, name });

export const cloudMoveFolder = (folderId: string, parentId: string | null) =>
  invoke<CloudFolder>("cloud_move_folder", { folderId, parentId });

export const cloudDeleteFolder = (folderId: string) =>
  invoke<void>("cloud_delete_folder", { folderId });

export const cloudListDocuments = (folderId?: string | null) =>
  invoke<CloudDocument[]>("cloud_list_documents", {
    folderId: folderId ?? null,
  });

export const cloudListShared = () => invoke<SharedItems>("cloud_list_shared");

export interface CloudFile {
  id: string;
  name: string;
  mime_type: string;
  folder_id: string | null;
  created_at: string;
}

export const cloudListFiles = (folderId?: string | null) =>
  invoke<CloudFile[]>("cloud_list_files", { folderId: folderId ?? null });

export const cloudDownloadFile = (fileId: string) =>
  invoke<string>("cloud_download_file", { fileId });

export const cloudDeleteFile = (fileId: string) =>
  invoke<void>("cloud_delete_file", { fileId });

export const cloudUploadFile = (path: string, folderId?: string | null) =>
  invoke<CloudFile>("cloud_upload_file", { path, folderId: folderId ?? null });

export const cloudRenameFile = (fileId: string, name: string) =>
  invoke<CloudFile>("cloud_rename_file", { fileId, name });

export const cloudMoveFile = (fileId: string, folderId: string | null) =>
  invoke<CloudFile>("cloud_move_file", { fileId, folderId });

export const cloudDownloadDocument = (documentId: string, parent: string) =>
  invoke<string>("cloud_download_document", { documentId, parent });

export const cloudDeleteDocument = (documentId: string) =>
  invoke<void>("cloud_delete_document", { documentId });

export const cloudCreateDocument = (path: string, title: string) =>
  invoke<string>("cloud_create_document", { path, title });

export const cloudMoveDocument = (documentId: string, folderId: string | null) =>
  invoke<CloudDocument>("cloud_move_document", { documentId, folderId });

export interface DocumentContent {
  id: string;
  title: string;
  role: string;
  hash: string;
  content: string;
}

export const cloudNewDocument = (title: string, folderId?: string | null) =>
  invoke<DocumentContent>("cloud_new_document", {
    title,
    folderId: folderId ?? null,
  });

export const cloudSyncDocument = (path: string) =>
  invoke<SyncReport>("cloud_sync_document", { path });

export const cloudResolveDocument = (
  path: string,
  content: string,
  serverHash: string,
) => invoke<void>("cloud_resolve_document", { path, content, serverHash });

export interface LinkedDocument {
  path: string;
  document_id: string;
  synced_at: string | null;
  sync_state: "synced" | "pending" | null;
}

export interface LinkedProject {
  path: string;
  cloud_project_id: string;
  synced_at: string | null;
  sync_state: "synced" | "pending" | null;
}

export const cloudLinkedDocuments = () =>
  invoke<LinkedDocument[]>("cloud_linked_documents");

export const cloudLinkedProjects = () =>
  invoke<LinkedProject[]>("cloud_linked_projects");

export const cloudDocumentLink = (path: string) =>
  invoke<DocumentLink | null>("cloud_document_link", { path });

export const cloudUnlinkDocument = (path: string) =>
  invoke<void>("cloud_unlink_document", { path });

export const cloudCreateProject = (name: string, folderId?: string | null) =>
  invoke<ProjectSummary>("cloud_create_project", {
    name,
    folderId: folderId ?? null,
  });

export const cloudDeleteProject = (cloudProjectId: string) =>
  invoke<void>("cloud_delete_project", { cloudProjectId });

export const cloudMoveProject = (
  cloudProjectId: string,
  folderId: string | null,
) => invoke<ProjectSummary>("cloud_move_project", { cloudProjectId, folderId });

export const cloudCloneProject = (
  cloudProjectId: string,
  projectName: string,
  parent: string,
) => invoke<SyncReport>("cloud_clone_project", { cloudProjectId, projectName, parent });

export const cloudLinkProject = (project: string, cloudProjectId?: string) =>
  invoke<SyncReport>("cloud_link_project", {
    project,
    cloudProjectId: cloudProjectId ?? null,
  });

export const cloudUnlinkProject = (project: string) =>
  invoke<void>("cloud_unlink_project", { project });

export const cloudPush = (project: string) =>
  invoke<SyncReport>("cloud_push", { project });

export const cloudPull = (project: string) =>
  invoke<SyncReport>("cloud_pull", { project });

export const cloudSync = (project: string) =>
  invoke<SyncReport>("cloud_sync", { project });

export const cloudResolveConflicts = (
  project: string,
  resolutions: Resolution[],
) => invoke<SyncReport>("cloud_resolve_conflicts", { project, resolutions });

export function errorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error && typeof error === "object" && "diagnostics" in error) {
    const failure = error as CompileFailure;
    return failure.diagnostics.map((d) => d.message).join("; ");
  }
  if (error instanceof Error) return error.message;
  return String(error);
}

export function isCompileFailure(error: unknown): error is CompileFailure {
  return Boolean(error && typeof error === "object" && "diagnostics" in error);
}
