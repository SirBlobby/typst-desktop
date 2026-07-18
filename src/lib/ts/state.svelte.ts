import * as api from "./api";
import type {
  Account,
  BrowseEntry,
  CompileResult,
  Conflict,
  Diagnostic,
  Settings,
  SpaceSummary,
  TargetInfo,
} from "./api";

export type Scope = "local" | "cloud";
export type View = "files" | "editor";
export type LspStatus = "off" | "starting" | "on" | "unavailable";

interface AppState {
  view: View;
  scope: Scope;
  settings: Settings | null;
  account: Account | null;

  currentDir: string;
  entries: BrowseEntry[];
  spaces: SpaceSummary[];

  target: TargetInfo | null;
  activePath: string | null;
  editorContent: string;
  dirty: boolean;
  compiled: CompileResult | null;
  diagnostics: Diagnostic[];
  compiling: boolean;
  lspStatus: LspStatus;

  syncing: boolean;
  conflicts: Conflict[];
  status: string;
  error: string;
  theme: "light" | "dark";
}

export const app = $state<AppState>({
  view: "files",
  scope: "local",
  settings: null,
  account: null,

  currentDir: "",
  entries: [],
  spaces: [],

  target: null,
  activePath: null,
  editorContent: "",
  dirty: false,
  compiled: null,
  diagnostics: [],
  compiling: false,
  lspStatus: "off",

  syncing: false,
  conflicts: [],
  status: "",
  error: "",
  theme: "light",
});

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

export function applyTheme(theme: "light" | "dark") {
  app.theme = theme;
  document.documentElement.dataset.theme = theme;
  localStorage.setItem("typst-desktop-theme", theme);
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
  const stored = localStorage.getItem("typst-desktop-theme");
  applyTheme(stored === "dark" ? "dark" : "light");

  try {
    app.settings = await api.getSettings();
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
      await refreshSpaces();
    } else {
      app.spaces = [];
    }
  } catch {
    app.account = null;
  }
}

export async function refreshSpaces() {
  try {
    app.spaces = await api.cloudListSpaces();
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
  if (app.dirty) await saveActiveFile();
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

  if (app.dirty && app.activePath) await saveActiveFile();

  try {
    const payload = await api.readTargetFile(app.target.path, file);
    app.activePath = file;
    app.editorContent = payload.is_text ? payload.content : "";
    app.dirty = false;
    if (payload.is_text) await compile();
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
    const result = await api.compileTarget(app.target.path, liveOverrides());
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

export async function saveAndCompile() {
  cancelScheduledCompile();
  await saveActiveFile();
  await compile();
}

export async function runSync(
  action: "sync" | "push" | "pull",
  project = app.target?.path,
) {
  if (!project) return;

  app.syncing = true;
  clearMessages();

  try {
    const report =
      action === "push"
        ? await api.cloudPush(project)
        : action === "pull"
          ? await api.cloudPull(project)
          : await api.cloudSync(project);

    app.conflicts = report.conflicts;

    if (report.conflicts.length > 0) {
      setError(`${report.conflicts.length} file(s) need conflict resolution`);
    } else {
      setStatus(summarize(report));
    }

    await refreshTarget();
    if (app.activePath) {
      const payload = await api.readTargetFile(project, app.activePath);
      if (payload.is_text) {
        app.editorContent = payload.content;
        app.dirty = false;
      }
    }
    await compile();
  } catch (error) {
    setError(error);
  } finally {
    app.syncing = false;
  }
}

function summarize(report: {
  pushed: string[];
  pulled: string[];
  merged: string[];
  deleted_local: string[];
  deleted_remote: string[];
}): string {
  const parts: string[] = [];
  if (report.pushed.length) parts.push(`${report.pushed.length} uploaded`);
  if (report.pulled.length) parts.push(`${report.pulled.length} downloaded`);
  if (report.merged.length) parts.push(`${report.merged.length} merged`);
  if (report.deleted_local.length)
    parts.push(`${report.deleted_local.length} removed locally`);
  if (report.deleted_remote.length)
    parts.push(`${report.deleted_remote.length} removed in cloud`);
  return parts.length
    ? `Sync complete: ${parts.join(", ")}`
    : "Already up to date";
}
