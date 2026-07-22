import hotkeys from "hotkeys-js";

// The default filter ignores any contenteditable target, which silently
// blocks every shortcut while the CodeMirror editor (contenteditable) has
// focus — exactly when these shortcuts are meant to fire. Only keep the
// exclusion for classic form fields (rename dialogs, prompts, etc).
hotkeys.filter = (event: KeyboardEvent) => {
  const target = (event.target as HTMLElement | null) ?? null;
  const tagName = target?.tagName;
  return tagName !== "INPUT" && tagName !== "TEXTAREA" && tagName !== "SELECT";
};

export interface HotkeyDef {
  id: string;
  group: string;
  label: string;
  defaultKeys: string;
}

export const HOTKEY_DEFS: HotkeyDef[] = [
  { id: "save", group: "Editor", label: "Save and compile", defaultKeys: "command+s,ctrl+s" },
  { id: "undo", group: "Editor", label: "Undo", defaultKeys: "command+z,ctrl+z" },
  {
    id: "redo",
    group: "Editor",
    label: "Redo",
    defaultKeys: "command+shift+z,ctrl+shift+z,ctrl+y",
  },
  {
    id: "toggleSidebar",
    group: "Editor",
    label: "Toggle file sidebar",
    defaultKeys: "command+shift+b,ctrl+shift+b",
  },
  { id: "bold", group: "Formatting", label: "Bold (toggle)", defaultKeys: "command+b,ctrl+b" },
  { id: "italic", group: "Formatting", label: "Italic (toggle)", defaultKeys: "command+i,ctrl+i" },
  {
    id: "underline",
    group: "Formatting",
    label: "Underline (toggle)",
    defaultKeys: "command+u,ctrl+u",
  },
  {
    id: "strikethrough",
    group: "Formatting",
    label: "Strikethrough (toggle)",
    defaultKeys: "command+shift+x,ctrl+shift+x",
  },
  { id: "link", group: "Formatting", label: "Insert link", defaultKeys: "command+k,ctrl+k" },
  {
    id: "numberedList",
    group: "Formatting",
    label: "Numbered list",
    defaultKeys: "command+shift+7,ctrl+shift+7",
  },
  {
    id: "bulletedList",
    group: "Formatting",
    label: "Bulleted list",
    defaultKeys: "command+shift+8,ctrl+shift+8",
  },
  ...[1, 2, 3, 4, 5, 6].map((level) => ({
    id: `heading${level}`,
    group: "Formatting",
    label: `Heading level ${level}`,
    defaultKeys: `command+alt+${level},ctrl+alt+${level}`,
  })),
];

const STORAGE_KEY = "hotkey-overrides";

function loadOverrides(): Record<string, string> {
  try {
    return JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}");
  } catch {
    return {};
  }
}

function saveOverrides(overrides: Record<string, string>) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(overrides));
}

export function keysFor(id: string): string {
  const def = HOTKEY_DEFS.find((entry) => entry.id === id);
  const overrides = loadOverrides();
  return overrides[id] ?? def?.defaultKeys ?? "";
}

export function isCustomized(id: string): boolean {
  return id in loadOverrides();
}

const registered = new Map<string, { keys: string; handler: (event: KeyboardEvent) => void }>();

export function registerHotkey(id: string, handler: (event: KeyboardEvent) => void) {
  const keys = keysFor(id);
  if (!keys) return;
  hotkeys(keys, handler);
  registered.set(id, { keys, handler });
}

export function unregisterAll() {
  for (const { keys } of registered.values()) hotkeys.unbind(keys);
  registered.clear();
}

export function rebindHotkey(id: string, newKeys: string) {
  const entry = registered.get(id);
  if (!entry) return;

  hotkeys.unbind(entry.keys);
  hotkeys(newKeys, entry.handler);
  registered.set(id, { keys: newKeys, handler: entry.handler });

  const def = HOTKEY_DEFS.find((item) => item.id === id);
  const overrides = loadOverrides();
  if (def && def.defaultKeys === newKeys) {
    delete overrides[id];
  } else {
    overrides[id] = newKeys;
  }
  saveOverrides(overrides);
}

export function resetHotkey(id: string) {
  const def = HOTKEY_DEFS.find((entry) => entry.id === id);
  if (def) rebindHotkey(id, def.defaultKeys);
}

const NAMED_KEYS: Record<string, string> = {
  " ": "space",
  Escape: "esc",
  ArrowUp: "up",
  ArrowDown: "down",
  ArrowLeft: "left",
  ArrowRight: "right",
};

export function comboFromEvent(event: KeyboardEvent): string | null {
  if (["Control", "Shift", "Alt", "Meta"].includes(event.key)) return null;

  const parts: string[] = [];
  if (event.ctrlKey) parts.push("ctrl");
  if (event.metaKey) parts.push("command");
  if (event.altKey) parts.push("alt");
  if (event.shiftKey) parts.push("shift");

  const key = event.key;
  const mainKey = key.length === 1 ? key.toLowerCase() : (NAMED_KEYS[key] ?? key.toLowerCase());
  parts.push(mainKey);

  return parts.join("+");
}
