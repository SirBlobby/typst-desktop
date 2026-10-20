import { listen } from "@tauri-apps/api/event";
import * as api from "./api";
import type { DeviceEvent, SyncReport } from "./api";
import {
  openCollabSession,
  closeCollabSession,
  type CollabSession,
} from "./yjs-client";
import {
  app,
  clearMessages,
  compile,
  refreshCloud,
  refreshTarget,
  setError,
  setStatus,
} from "./state.svelte";

type SyncAction = "sync" | "push" | "pull";

const LIVE_CONNECT_TIMEOUT_MS = 10000;
const PUSH_AFTER_SAVE_MS = 2000;

let pendingSession: CollabSession | null = null;
let pushTimer: ReturnType<typeof setTimeout> | null = null;
let syncTimer: ReturnType<typeof setInterval> | null = null;

function isLinkedTarget() {
  return (
    !!app.account &&
    !!app.target &&
    (!!app.target.cloud_project_id || !!app.documentLink)
  );
}

function hasLiveSession() {
  return !!app.collab || app.collabIntent;
}

export async function initWsSync() {
  await listen<string>("cloud://ws-status", (event) => {
    app.wsStatus = event.payload;
  });

  await listen<DeviceEvent>("cloud://sync-event", (event) => {
    handleDeviceEvent(event.payload);
  });

  app.wsStatus = await api.cloudWsStatus().catch(() => "offline");
}

function handleDeviceEvent(event: DeviceEvent) {
  const linkedProject = app.target?.cloud_project_id;
  const linkedDocument = app.documentLink?.document_id;

  const matchesOpenTarget =
    (event.kind === "project" &&
      !!event.project_id &&
      event.project_id === linkedProject) ||
    (event.kind === "document" &&
      !!event.document_id &&
      event.document_id === linkedDocument);

  if (matchesOpenTarget) {
    autoSync();
  } else if (app.scope === "cloud") {
    refreshCloud();
  }
}

function syncRequest(path: string, isDocument: boolean, action: SyncAction) {
  if (isDocument) return api.cloudSyncDocument(path);
  if (action === "push") return api.cloudPush(path);
  if (action === "pull") return api.cloudPull(path);
  return api.cloudSync(path);
}

export async function runSync(
  action: SyncAction,
  project = app.target?.path,
  quiet = false,
) {
  if (!project || app.syncing) return;

  app.syncing = true;
  if (!quiet) clearMessages();

  try {
    const report = await syncRequest(project, !!app.documentLink, action);
    app.conflicts = report.conflicts;

    if (report.conflicts.length > 0) {
      setError(`${report.conflicts.length} file(s) need conflict resolution`);
    } else if (!quiet) {
      setStatus(summarize(report));
    }

    if (app.target?.path !== project) return;
    await refreshTarget();
    if (app.activePath && !hasLiveSession()) {
      const payload = await api.readTargetFile(project, app.activePath);
      if (payload.is_text && payload.content !== app.editorContent) {
        app.editorContent = payload.content;
        app.dirty = false;
      }
    }
    await compile();
  } catch (error) {
    if (!quiet) setError(error);
  } finally {
    app.syncing = false;
  }
}

export async function autoSync() {
  if (!isLinkedTarget() || hasLiveSession()) return;
  if (app.syncing || app.conflicts.length > 0) return;
  await runSync("sync", app.target?.path, true);
}

export async function syncBeforeOpening() {
  if (!isLinkedTarget()) return;
  await runSync("sync", app.target?.path, true);
}

export function syncAfterSave() {
  if (!isLinkedTarget() || hasLiveSession()) return;
  if (pushTimer) clearTimeout(pushTimer);
  pushTimer = setTimeout(() => {
    pushTimer = null;
    autoSync();
  }, PUSH_AFTER_SAVE_MS);
}

function syncInBackground(path: string, isDocument: boolean) {
  if (!app.account) return;
  syncRequest(path, isDocument, "sync").catch(() => {});
}

export function restartAutoSync() {
  if (syncTimer) {
    clearInterval(syncTimer);
    syncTimer = null;
  }

  const seconds = app.settings?.sync_seconds ?? 0;
  if (seconds <= 0) return;

  syncTimer = setInterval(autoSync, seconds * 1000);
}

function waitForFirstSync(session: CollabSession) {
  return new Promise<boolean>((resolve) => {
    const timer = setTimeout(() => resolve(false), LIVE_CONNECT_TIMEOUT_MS);
    session.provider.once("sync", (isSynced: boolean) => {
      clearTimeout(timer);
      resolve(isSynced);
    });
  });
}

function bindSession(session: CollabSession) {
  app.collab = session;
  app.collabStatus = session.provider.wsconnected ? "connected" : "connecting";
  session.provider.on("status", (event: { status: string }) => {
    app.collabStatus =
      event.status === "connected"
        ? "connected"
        : event.status === "connecting"
          ? "connecting"
          : "offline";
  });
}

function replaceSessionText(session: CollabSession, content: string) {
  const sharedText = session.text;
  sharedText.doc?.transact(() => {
    sharedText.delete(0, sharedText.length);
    sharedText.insert(0, content);
  });
}

export async function openLiveSession(file: string, diskContent: string) {
  if (!app.target || !app.account || !app.settings?.device_token) return;

  let roomId: string | null = null;
  try {
    roomId = await api.cloudRoomId(app.target.path, file);
  } catch {
    roomId = null;
  }
  if (!roomId || app.activePath !== file) return;

  app.collabIntent = true;
  app.collabStatus = "connecting";

  const session = openCollabSession(
    app.settings.server_url,
    app.settings.device_token,
    roomId,
  );
  const synced = await waitForFirstSync(session);

  if (app.activePath !== file || !app.collabIntent) {
    closeCollabSession(session);
    return;
  }

  if (!synced) {
    closeCollabSession(session);
    app.collabIntent = false;
    app.collabStatus = null;
    autoSync();
    return;
  }

  const remoteText = session.text.toString();
  if (remoteText === "" && diskContent !== "") {
    replaceSessionText(session, diskContent);
  } else if (remoteText !== diskContent) {
    app.collabConflict = {
      path: file,
      local_text: diskContent,
      remote_text: remoteText,
      merged_text: remoteText,
      server_hash: "",
      auto_merged: false,
      binary: false,
    };
    pendingSession = session;
    return;
  }

  bindSession(session);
}

export function closeLiveSession() {
  const target = app.target;
  const wasLive = !!app.collab;

  closeCollabSession(app.collab);
  closeCollabSession(pendingSession);
  pendingSession = null;
  app.collab = null;
  app.collabIntent = false;
  app.collabStatus = null;
  app.collabConflict = null;

  if (wasLive && target) syncInBackground(target.path, !!app.documentLink);
}

export function resolveCollabConflict(content: string) {
  const session = pendingSession;
  pendingSession = null;
  app.collabConflict = null;
  if (!session) return;

  replaceSessionText(session, content);
  bindSession(session);
}

export function cancelCollabConflict() {
  closeCollabSession(pendingSession);
  pendingSession = null;
  app.collabConflict = null;
  app.collabIntent = false;
  app.collabStatus = null;
}

function summarize(report: SyncReport): string {
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
