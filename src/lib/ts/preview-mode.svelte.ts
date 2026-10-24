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
