export type PreviewMode = "split" | "editor" | "preview";

export const previewModeOptions: { value: PreviewMode; label: string; icon: string }[] = [
  { value: "split", label: "Editor and preview", icon: "ph:columns" },
  { value: "editor", label: "Editor only", icon: "ph:code" },
  { value: "preview", label: "Preview only", icon: "ph:eye" },
];

const PREVIEW_MODE_KEY = "editor-preview-mode";

function isPreviewMode(value: string | null): value is PreviewMode {
  return value === "split" || value === "editor" || value === "preview";
}

function loadPreviewMode(): PreviewMode {
  if (typeof localStorage === "undefined") return "split";
  const saved = localStorage.getItem(PREVIEW_MODE_KEY);
  return isPreviewMode(saved) ? saved : "split";
}

export const previewMode = $state<{ current: PreviewMode }>({ current: loadPreviewMode() });

export function setPreviewMode(mode: PreviewMode) {
  previewMode.current = mode;
  localStorage.setItem(PREVIEW_MODE_KEY, mode);
}

export const MIN_SPLIT = 0.2;
export const MAX_SPLIT = 0.8;
export const DEFAULT_SPLIT = 0.5;

const SPLIT_RATIO_KEY = "editor-split-ratio";

export function clampSplit(ratio: number) {
  return Math.min(MAX_SPLIT, Math.max(MIN_SPLIT, ratio));
}

function loadSplitRatio(): number {
  if (typeof localStorage === "undefined") return DEFAULT_SPLIT;
  const saved = Number.parseFloat(localStorage.getItem(SPLIT_RATIO_KEY) ?? "");
  return Number.isFinite(saved) ? clampSplit(saved) : DEFAULT_SPLIT;
}

export const splitRatio = $state<{ current: number }>({ current: loadSplitRatio() });

export function setSplitRatio(ratio: number) {
  splitRatio.current = clampSplit(ratio);
  localStorage.setItem(SPLIT_RATIO_KEY, String(splitRatio.current));
}
