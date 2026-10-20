import { invoke } from "@tauri-apps/api/core";
import { writable } from "svelte/store";

export const reviewPanelOpenStore = writable(false);
export const historyOpenStore = writable(false);

export function reviewRequest<T>(method: string, path: string, body?: unknown): Promise<T> {
  return invoke<T>("cloud_review_request", { method, path, body: body ?? null });
}

const avatarCache = new Map<string, Promise<string | null>>();

export function avatarFor(userId: string): Promise<string | null> {
  let pending = avatarCache.get(userId);
  if (!pending) {
    pending = invoke<string | null>("cloud_user_avatar", { userId }).catch(() => null);
    avatarCache.set(userId, pending);
  }
  return pending;
}
