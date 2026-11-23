import * as api from "./api";
import type { ClickTarget, PagePoint } from "./api";
import { app } from "./state.svelte";

export interface CompileInput {
  targetPath: string;
  activePath: string | null;
  editorContent: string;
  entrypoint?: string;
  overrides?: Record<string, string>;
}

let lastCompileInput: CompileInput | null = null;

export function recordCompileInput(input: CompileInput) {
  lastCompileInput = input;
}

function matchingCompileInput(): CompileInput | null {
  if (!app.target || !lastCompileInput) return null;
  const unchanged =
    lastCompileInput.targetPath === app.target.path &&
    lastCompileInput.activePath === app.activePath &&
    lastCompileInput.editorContent === app.editorContent;
  return unchanged ? lastCompileInput : null;
}

export async function previewPointForCursor(offset: number): Promise<PagePoint | null> {
  if (!app.activePath || !/\.typ$/i.test(app.activePath)) return null;
  const input = matchingCompileInput();
  if (!input) return null;
  const positions = await api
    .syncCursor(input.targetPath, app.activePath, offset, input.entrypoint, input.overrides)
    .catch(() => []);
  return positions[0] ?? null;
}

export async function sourceForPreviewClick(point: PagePoint): Promise<ClickTarget | null> {
  const input = matchingCompileInput();
  if (!input) return null;
  return api.syncClick(input.targetPath, point, input.entrypoint, input.overrides).catch(() => null);
}
