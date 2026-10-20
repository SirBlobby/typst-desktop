import * as api from "./api";
import {
  closeLiveSession,
  initWsSync,
  openLiveSession,
  restartAutoSync,
  syncAfterSave,
  syncBeforeOpening,
} from "./cloud-sync";
import type { CollabSession } from "./yjs-client";
import type {
  Account,
  BrowseEntry,
  CloudDocument,
  CloudFile,
  CloudFolder,
  CompileResult,
  Conflict,
  Diagnostic,
  DocumentLink,
  LinkedDocument,
  LinkedProject,
  ProjectSummary,
  Settings,
  TargetInfo,
} from "./api";

export type Scope = "local" | "cloud";
export type View = "files" | "editor";
export type LspStatus = "off" | "starting" | "on" | "unavailable";
export type ThemePreference = "light" | "dark" | "system";
export type TextScale = "small" | "default" | "large" | "xlarge";
export type ContrastLevel = "normal" | "high";
export type ColorTheme = "catppuccin" | "cerberus" | "archlinux";

export const colorThemes: { id: ColorTheme; label: string; accent: string; icon: string }[] = [
  { id: "cerberus", label: "Cerberus", accent: "#4f46e5", icon: "mdi:dog" },
  { id: "catppuccin", label: "Catppuccin", accent: "#1e66f5", icon: "mdi:cat" },
  { id: "archlinux", label: "Arch Linux", accent: "#0a6ea8", icon: "mdi:penguin" },
];

interface AppState {
  view: View;
  scope: Scope;
  settings: Settings | null;
  account: Account | null;

  currentDir: string;
  entries: BrowseEntry[];
  cloudProjects: ProjectSummary[];
  cloudFolder: string | null | "shared";
  cloudFolders: CloudFolder[];
  cloudFolderTree: CloudFolder[];
  cloudDocuments: CloudDocument[];
  cloudFiles: CloudFile[];
  cloudLoading: boolean;
  cloudOffline: boolean;
  wsStatus: string;
  linkedDocuments: LinkedDocument[];
  linkedProjects: LinkedProject[];
  documentLink: DocumentLink | null;

  target: TargetInfo | null;
  activePath: string | null;
  editorContent: string;
  dirty: boolean;
  compiled: CompileResult | null;
  diagnostics: Diagnostic[];
  compiling: boolean;
  lspStatus: LspStatus;

  download: DownloadProgress | null;
  syncing: boolean;
  conflicts: Conflict[];
  collab: CollabSession | null;
  collabIntent: boolean;
  collabStatus: "connecting" | "connected" | "offline" | null;
  collabConflict: Conflict | null;
  status: string;
  error: string;
  theme: "light" | "dark";
  themePreference: ThemePreference;
  colorTheme: ColorTheme;
  accent: string | null;
  textScale: TextScale;
  reduceMotion: boolean;
  contrast: ContrastLevel;
}

export const app = $state<AppState>({
  view: "files",
  scope: "local",
  settings: null,
  account: null,

  currentDir: "",
  entries: [],
  cloudProjects: [],
  cloudFolder: null,
  cloudFolders: [],
  cloudFolderTree: [],
  cloudDocuments: [],
  cloudFiles: [],
  cloudLoading: false,
  cloudOffline: false,
  wsStatus: "offline",
  linkedDocuments: [],
  linkedProjects: [],
  documentLink: null,

  target: null,
  activePath: null,
  editorContent: "",
  dirty: false,
  compiled: null,
  diagnostics: [],
  compiling: false,
  lspStatus: "off",

  download: null,
  syncing: false,
  conflicts: [],
  collab: null,
  collabIntent: false,
  collabStatus: null,
  collabConflict: null,
  status: "",
  error: "",
  theme: "light",
  themePreference: "light",
  colorTheme: "catppuccin",
  accent: null,
  textScale: "default",
  reduceMotion: false,
  contrast: "normal",
});

export interface DownloadProgress {
  label: string;
  current: number;
  total: number;
  done: boolean;
}

let downloadClearTimer: ReturnType<typeof setTimeout> | null = null;

export function trackDownload(progress: DownloadProgress) {
  if (downloadClearTimer) {
    clearTimeout(downloadClearTimer);
    downloadClearTimer = null;
  }

  app.download = progress;

  if (progress.done) {
    downloadClearTimer = setTimeout(() => {
      app.download = null;
      downloadClearTimer = null;
    }, 1200);
  }
}

export function setError(error: unknown) {
  app.error = api.errorMessage(error);
  app.status = "";
}

export function setStatus(message: string) {
  app.status = message;
  app.error = "";
}

export function clearMessages() {
  app.status = "";
  app.error = "";
}

const THEME_KEY = "typst-desktop-theme";
const COLOR_THEME_KEY = "typst-desktop-color-theme";
const ACCENT_KEY = "typst-desktop-accent";
const TEXT_SCALE_KEY = "typst-desktop-text-scale";
const REDUCE_MOTION_KEY = "typst-desktop-reduce-motion";
const CONTRAST_KEY = "typst-desktop-contrast";

const TEXT_SCALE_PX: Record<TextScale, number> = {
  small: 14,
  default: 16,
  large: 18,
  xlarge: 20,
};

let systemThemeQuery: MediaQueryList | null = null;

function resolveTheme(preference: ThemePreference): "light" | "dark" {
  if (preference !== "system") return preference;

  if (!systemThemeQuery) {
    systemThemeQuery = window.matchMedia("(prefers-color-scheme: dark)");
    systemThemeQuery.addEventListener("change", () => {
      if (app.themePreference === "system") applyTheme("system");
    });
  }

  return systemThemeQuery.matches ? "dark" : "light";
}

export function applyTheme(preference: ThemePreference) {
  app.themePreference = preference;
  app.theme = resolveTheme(preference);
  document.documentElement.dataset.theme = app.theme;
  localStorage.setItem(THEME_KEY, preference);

  if (app.accent) applyAccent(app.accent);
}

export function applyColorTheme(theme: ColorTheme) {
  app.colorTheme = theme;
  document.documentElement.dataset.colorTheme = theme;
  localStorage.setItem(COLOR_THEME_KEY, theme);
  applyAccent(null);
}

function hexToRgb(hex: string): [number, number, number] {
  const value = parseInt(hex.replace("#", ""), 16);
  return [(value >> 16) & 255, (value >> 8) & 255, value & 255];
}

function rgbToHsl(r: number, g: number, b: number): [number, number, number] {
  r /= 255;
  g /= 255;
  b /= 255;
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;
  let h = 0;
  let s = 0;

  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    switch (max) {
      case r:
        h = (g - b) / d + (g < b ? 6 : 0);
        break;
      case g:
        h = (b - r) / d + 2;
        break;
      default:
        h = (r - g) / d + 4;
    }
    h /= 6;
  }

  return [h * 360, s * 100, l * 100];
}

function hslToHex(h: number, s: number, l: number): string {
  s /= 100;
  l /= 100;
  const k = (n: number) => (n + h / 30) % 12;
  const a = s * Math.min(l, 1 - l);
  const f = (n: number) =>
    l - a * Math.max(-1, Math.min(k(n) - 3, 9 - k(n), 1));
  const toHex = (n: number) =>
    Math.round(n * 255)
      .toString(16)
      .padStart(2, "0");
  return `#${toHex(f(0))}${toHex(f(8))}${toHex(f(4))}`;
}

function accentSoft(hex: string, dark: boolean): string {
  const [r, g, b] = hexToRgb(hex);
  const [h, s] = rgbToHsl(r, g, b);
  return dark
    ? hslToHex(h, Math.min(s, 55), 18)
    : hslToHex(h, Math.min(s, 70), 92);
}

export function applyAccent(color: string | null) {
  app.accent = color;
  const root = document.documentElement.style;

  if (color) {
    root.setProperty("--color-accent", color);
    root.setProperty("--color-accent-soft", accentSoft(color, app.theme === "dark"));
    localStorage.setItem(ACCENT_KEY, color);
  } else {
    root.removeProperty("--color-accent");
    root.removeProperty("--color-accent-soft");
    localStorage.removeItem(ACCENT_KEY);
  }
}

export function applyTextScale(scale: TextScale) {
  app.textScale = scale;
  document.documentElement.style.fontSize = `${TEXT_SCALE_PX[scale]}px`;
  localStorage.setItem(TEXT_SCALE_KEY, scale);
}

export function applyReduceMotion(enabled: boolean) {
  app.reduceMotion = enabled;
  if (enabled) {
    document.documentElement.dataset.reduceMotion = "true";
  } else {
    delete document.documentElement.dataset.reduceMotion;
  }
  localStorage.setItem(REDUCE_MOTION_KEY, String(enabled));
}

export function applyContrast(level: ContrastLevel) {
  app.contrast = level;
  if (level === "high") {
    document.documentElement.dataset.contrast = "high";
  } else {
    delete document.documentElement.dataset.contrast;
  }
  localStorage.setItem(CONTRAST_KEY, level);
}

export function breadcrumbs(): { name: string; path: string }[] {
  if (!app.currentDir) return [];
  const segments = app.currentDir.split("/");
  return segments.map((name, index) => ({
    name,
    path: segments.slice(0, index + 1).join("/"),
  }));
}

export async function bootstrap() {
  const storedTheme = localStorage.getItem(THEME_KEY);
  applyTheme(
    storedTheme === "dark" || storedTheme === "light" || storedTheme === "system"
      ? storedTheme
      : "light",
  );

  const storedColorTheme = localStorage.getItem(COLOR_THEME_KEY);
  const validColorTheme = colorThemes.some((entry) => entry.id === storedColorTheme);
  const initialColorTheme: ColorTheme = validColorTheme ? (storedColorTheme as ColorTheme) : "catppuccin";
  document.documentElement.dataset.colorTheme = initialColorTheme;
  app.colorTheme = initialColorTheme;

  const storedAccent = localStorage.getItem(ACCENT_KEY);
  if (storedAccent) applyAccent(storedAccent);

  const storedTextScale = localStorage.getItem(TEXT_SCALE_KEY);
  applyTextScale(
    storedTextScale === "small" ||
      storedTextScale === "large" ||
      storedTextScale === "xlarge"
      ? storedTextScale
      : "default",
  );

  applyReduceMotion(localStorage.getItem(REDUCE_MOTION_KEY) === "true");
  applyContrast(localStorage.getItem(CONTRAST_KEY) === "high" ? "high" : "normal");

  try {
    app.settings = await api.getSettings();
    restartAutoSync();
    await initWsSync();
    if (app.settings?.device_token) {
      api.cloudWsStart().catch(() => {});
    }
    await browseTo("");
    await refreshAccount();
  } catch (error) {
    setError(error);
  }
}

export async function browseTo(path: string) {
  try {
    app.entries = await api.browseWorkspace(path);
    app.currentDir = path;
    clearMessages();
  } catch (error) {
    setError(error);
  }
}

export async function refreshEntries() {
  await browseTo(app.currentDir);
}

export async function refreshAccount() {
  try {
    app.account = await api.cloudAccount();
    if (app.account) {
      await refreshCloudProjects();
    } else {
      app.cloudProjects = [];
      app.scope = "local";
    }
  } catch {
    app.account = null;
    app.scope = "local";
  }
}

export async function refreshCloudProjects() {
  try {
    app.cloudProjects = await api.cloudListProjects();
  } catch (error) {
    setError(error);
  }
}

interface CloudSnapshot {
  folders: CloudFolder[];
  documents: CloudDocument[];
  projects: ProjectSummary[];
  files: CloudFile[];
}

function cloudCacheKey() {
  return `cloud:${app.cloudFolder ?? "root"}`;
}

function applyCloudSnapshot(snapshot: CloudSnapshot) {
  if (app.cloudFolder === "shared") {
    app.cloudDocuments = snapshot.documents;
    app.cloudProjects = snapshot.projects;
    app.cloudFolders = [];
    app.cloudFiles = [];
  } else {
    app.cloudFolderTree = snapshot.folders;
    app.cloudFolders = snapshot.folders.filter(
      (folder) => (folder.parent_id ?? null) === app.cloudFolder,
    );
    app.cloudDocuments = snapshot.documents;
    app.cloudProjects = snapshot.projects.filter(
      (project) =>
        project.role !== "owner" ||
        (project.folder_id ?? null) === app.cloudFolder,
    );
    app.cloudFiles = snapshot.files;
  }
}

export async function refreshCloud() {
  if (!app.account) return;

  app.cloudLoading = true;
  const cacheKey = cloudCacheKey();

  try {
    app.linkedDocuments = await api.cloudLinkedDocuments().catch(() => []);
    app.linkedProjects = await api.cloudLinkedProjects().catch(() => []);

    let snapshot: CloudSnapshot;
    if (app.cloudFolder === "shared") {
      const shared = await api.cloudListShared();
      snapshot = { folders: [], documents: shared.documents, projects: shared.projects, files: [] };
    } else {
      const [folders, documents, projects, files] = await Promise.all([
        api.cloudListFolders(),
        api.cloudListDocuments(app.cloudFolder),
        api.cloudListProjects(),
        api.cloudListFiles(app.cloudFolder),
      ]);
      snapshot = { folders, documents, projects, files };
    }

    applyCloudSnapshot(snapshot);
    app.cloudOffline = false;
    api.saveCloudCache(cacheKey, JSON.stringify(snapshot)).catch(() => {});
  } catch (error) {
    const cached = await api.getCloudCache(cacheKey).catch(() => null);
    if (cached) {
      applyCloudSnapshot(JSON.parse(cached));
      app.cloudOffline = true;
    } else {
      setError(error);
    }
  } finally {
    app.cloudLoading = false;
  }
}

export function cloudBreadcrumbs(): CloudFolder[] {
  if (app.cloudFolder === "shared" || app.cloudFolder === null) return [];

  const byId = new Map(app.cloudFolderTree.map((folder) => [folder.id, folder]));
  const trail: CloudFolder[] = [];

  let current = byId.get(app.cloudFolder);
  while (current) {
    trail.unshift(current);
    current = current.parent_id ? byId.get(current.parent_id) : undefined;
  }

  return trail;
}

export async function openCloudFolder(id: string | null | "shared") {
  app.cloudFolder = id;
  await refreshCloud();
}

export async function downloadDocument(documentId: string, title: string) {
  try {
    const path = await api.cloudDownloadDocument(documentId, app.currentDir);
    await refreshCloud();
    setStatus(`Downloaded '${title}' to this device`);
    return path;
  } catch (error) {
    setError(error);
    return null;
  }
}

export function linkedDocument(documentId: string) {
  return app.linkedDocuments.find(
    (linked) => linked.document_id === documentId,
  );
}

export async function downloadCloudFile(fileId: string, name: string) {
  try {
    await api.cloudDownloadFile(fileId);
    setStatus(`'${name}' added to your shared assets`);
  } catch (error) {
    setError(error);
  }
}

export function linkedProject(cloudProjectId: string) {
  return app.linkedProjects.find(
    (linked) => linked.cloud_project_id === cloudProjectId,
  );
}

export async function removeDownloadedDocument(path: string) {
  try {
    await api.deleteEntry(path);
    await api.cloudUnlinkDocument(path);
    await refreshCloud();
    setStatus("Removed from this device");
  } catch (error) {
    setError(error);
  }
}

export async function openTarget(path: string) {
  try {
    const target = await api.targetInfo(path);
    app.target = target;
    app.view = "editor";
    app.activePath = null;
    app.editorContent = "";
    app.dirty = false;
    app.compiled = null;
    app.diagnostics = [];
    app.lspStatus = "off";
    clearMessages();

    app.documentLink = target.standalone
      ? await api.cloudDocumentLink(path).catch(() => null)
      : null;

    await syncBeforeOpening();

    const preferred =
      target.files.find((file) => file.path === target.entrypoint) ??
      target.files.find((file) => file.path.endsWith(".typ")) ??
      target.files[0];

    if (preferred) await openFile(preferred.path);
  } catch (error) {
    setError(error);
  }
}

export async function closeTarget() {
  cancelScheduledCompile();
  cancelAutosave();
  if (app.dirty) await saveActiveFile();
  closeLiveSession();
  app.view = "files";
  app.target = null;
  app.activePath = null;
  app.editorContent = "";
  app.compiled = null;
  app.diagnostics = [];
  app.lspStatus = "off";
  await refreshEntries();
}

export async function refreshTarget() {
  if (!app.target) return;
  try {
    app.target = await api.targetInfo(app.target.path);
  } catch (error) {
    setError(error);
  }
}

export async function openFile(file: string) {
  if (!app.target) return;

  cancelScheduledCompile();
  cancelAutosave();

  if (app.dirty && app.activePath) await saveActiveFile();
  closeLiveSession();

  try {
    const payload = await api.readTargetFile(app.target.path, file);
    app.activePath = file;
    app.editorContent = payload.is_text ? payload.content : "";
    app.dirty = false;
    if (payload.is_text) await compile();
    if (payload.is_text) await openLiveSession(file, app.editorContent);
  } catch (error) {
    setError(error);
  }
}

export async function saveActiveFile() {
  if (!app.target || !app.activePath) return;
  try {
    await api.writeTargetFile(
      app.target.path,
      app.activePath,
      app.editorContent,
    );
    app.dirty = false;
    syncAfterSave();
  } catch (error) {
    setError(error);
  }
}

function liveOverrides(): Record<string, string> | undefined {
  if (!app.dirty || !app.activePath) return undefined;
  return { [app.activePath]: app.editorContent };
}

let compileRunning = false;
let compileQueued = false;

export async function compile() {
  if (!app.target) return;

  if (compileRunning) {
    compileQueued = true;
    return;
  }

  compileRunning = true;
  app.compiling = true;

  try {
    const previewFile =
      app.activePath && app.activePath.toLowerCase().endsWith(".typ")
        ? app.activePath
        : undefined;
    const result = await api.compileTarget(
      app.target.path,
      previewFile,
      liveOverrides(),
    );
    app.compiled = result;
    app.diagnostics = result.diagnostics;
  } catch (error) {
    if (api.isCompileFailure(error)) {
      app.diagnostics = error.diagnostics;
    } else {
      setError(error);
    }
  } finally {
    compileRunning = false;
    app.compiling = false;

    if (compileQueued) {
      compileQueued = false;
      await compile();
    }
  }
}

const COMPILE_DEBOUNCE_MS = 400;
let compileTimer: ReturnType<typeof setTimeout> | null = null;

export function scheduleCompile() {
  if (compileTimer) clearTimeout(compileTimer);
  compileTimer = setTimeout(() => {
    compileTimer = null;
    compile();
  }, COMPILE_DEBOUNCE_MS);
}

export function cancelScheduledCompile() {
  if (compileTimer) {
    clearTimeout(compileTimer);
    compileTimer = null;
  }
}

let autosaveTimer: ReturnType<typeof setTimeout> | null = null;

const COLLAB_AUTOSAVE_SECONDS = 5;

export function scheduleAutosave() {
  if (autosaveTimer) clearTimeout(autosaveTimer);

  const seconds = app.collabIntent
    ? Math.min(app.settings?.autosave_seconds || COLLAB_AUTOSAVE_SECONDS, COLLAB_AUTOSAVE_SECONDS)
    : (app.settings?.autosave_seconds ?? 0);
  if (seconds <= 0) return;

  autosaveTimer = setTimeout(() => {
    autosaveTimer = null;
    if (app.dirty) saveActiveFile();
  }, seconds * 1000);
}

export function cancelAutosave() {
  if (autosaveTimer) {
    clearTimeout(autosaveTimer);
    autosaveTimer = null;
  }
}

export async function saveAndCompile() {
  cancelScheduledCompile();
  cancelAutosave();
  await saveActiveFile();
  await compile();
}

export {
  autoSync,
  restartAutoSync,
  runSync,
  resolveCollabConflict,
  cancelCollabConflict,
} from "./cloud-sync";
