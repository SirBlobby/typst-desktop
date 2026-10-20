import * as api from "./api";
import { app, openCloudFolder } from "./state.svelte";

const SAVED_SPACE_KEY = "typst-desktop-cloud-space";

function savedSpace(): string | null {
  try {
    return localStorage.getItem(SAVED_SPACE_KEY);
  } catch {
    return null;
  }
}

function rememberSpace(teamId: string | null) {
  try {
    if (teamId) localStorage.setItem(SAVED_SPACE_KEY, teamId);
    else localStorage.removeItem(SAVED_SPACE_KEY);
  } catch {}
}

async function applySpace(teamId: string | null) {
  await api.cloudSetSpace(teamId);
  app.cloudSpace = teamId;
  rememberSpace(teamId);
}

export async function refreshCloudSpaces() {
  app.cloudSpaces = await api.cloudListSpaces().catch(() => []);
  const wanted = app.cloudSpace ?? savedSpace();
  const stillMember = app.cloudSpaces.some((space) => space.id === wanted);
  await applySpace(stillMember ? wanted : null);
}

export async function clearCloudSpaces() {
  app.cloudSpaces = [];
  await applySpace(null);
}

export async function openCloudSpace(teamId: string | null, folder: null | "shared" = null) {
  await applySpace(teamId);
  app.scope = "cloud";
  await openCloudFolder(folder);
}

export function activeCloudSpace(): api.CloudSpace | null {
  return app.cloudSpaces.find((space) => space.id === app.cloudSpace) ?? null;
}

export function canChangeActiveSpace(): boolean {
  return activeCloudSpace()?.role !== "viewer";
}
