import type { EditorView } from "@codemirror/view";
import { insertText, prefixLines, wrapSelection } from "$lib/ts/editor-actions";
import { insertQuote, openFind, openReplace } from "$lib/ts/editor/insert";
import { insertDialogStore, insertOptions } from "$lib/ts/editor/insert-dialog";

export interface ToolbarItem {
  icon: string;
  label: string;
  run: () => void;
}

export interface ToolbarMenu {
  icon: string;
  label: string;
  items: ToolbarItem[];
}

export interface ToolbarGroup {
  id: string;
  label: string;
  icon: string;
  items: ToolbarItem[];
  menu?: ToolbarMenu;
  extraWidth?: number;
}

const BUTTON_WIDTH = 30;
const MENU_BUTTON_WIDTH = 42;
const DIVIDER_WIDTH = 11;
const COLLAPSED_GROUP_WIDTH = 44;
const HEADING_SELECT_WIDTH = 132;
const TOOLBAR_PADDING = 20;
const FONT_SELECT_WIDTH = 150;

const COLLAPSE_ORDER = ["find", "math", "lists", "document", "insert", "format"];

export function buildToolbarGroups(
  view: EditorView | null,
  onassets: () => void,
  onpagesettings: () => void,
  oncitations: () => void,
): ToolbarGroup[] {
  return [
    {
      id: "format",
      label: "Formatting",
      icon: "ph:text-aa",
      items: [
        { icon: "ph:text-b", label: "Bold", run: () => wrapSelection(view, "*", "*", "bold") },
        { icon: "ph:text-italic", label: "Italic", run: () => wrapSelection(view, "_", "_", "italic") },
        { icon: "ph:code", label: "Raw", run: () => wrapSelection(view, "`", "`", "code") },
        { icon: "ph:quotes", label: "Quote", run: () => view && insertQuote(view) },
      ],
    },
    {
      id: "math",
      label: "Math",
      icon: "ph:sigma",
      items: [
        { icon: "ph:sigma", label: "Inline math", run: () => wrapSelection(view, "$", "$", "x = y") },
        { icon: "ph:function", label: "Block math", run: () => wrapSelection(view, "$ \n  ", "\n$", "x = y") },
      ],
    },
    {
      id: "lists",
      label: "Lists",
      icon: "ph:list-bullets",
      items: [
        { icon: "ph:list-bullets", label: "Bullet list", run: () => prefixLines(view, "- ", "List item") },
        { icon: "ph:list-numbers", label: "Numbered list", run: () => prefixLines(view, "+ ", "Numbered item") },
      ],
    },
    {
      id: "insert",
      label: "Insert",
      icon: "ph:plus-square",
      items: [
        { icon: "ph:link", label: "Link", run: () => insertText(view, '#link("https://")[text]') },
        {
          icon: "ph:image-square",
          label: "Figure",
          run: () => insertText(view, '#figure(\n  image("file.png"),\n  caption: [Caption],\n)'),
        },
        { icon: "ph:images", label: "Images and fonts", run: onassets },
        { icon: "ph:quotes", label: "Citations", run: oncitations },
      ],
      menu: {
        icon: "ph:plus-square",
        label: "Insert",
        items: insertOptions.map((option) => ({
          icon: option.icon,
          label: option.label,
          run: () => insertDialogStore.set(option.dialog),
        })),
      },
    },
    {
      id: "find",
      label: "Find",
      icon: "ph:magnifying-glass",
      items: [
        { icon: "ph:magnifying-glass", label: "Find", run: () => view && openFind(view) },
        { icon: "ph:swap", label: "Search and replace", run: () => view && openReplace(view) },
      ],
    },
    {
      id: "document",
      label: "Document",
      icon: "ph:file-text",
      items: [{ icon: "ph:file-text", label: "Page settings", run: onpagesettings }],
      extraWidth: FONT_SELECT_WIDTH,
    },
  ];
}

export function allGroupItems(group: ToolbarGroup): ToolbarItem[] {
  return [...group.items, ...(group.menu?.items ?? [])];
}

function expandedWidth(group: ToolbarGroup): number {
  const menuWidth = group.menu ? MENU_BUTTON_WIDTH : 0;
  return group.items.length * BUTTON_WIDTH + menuWidth + DIVIDER_WIDTH + (group.extraWidth ?? 0);
}

export function collapsedGroupIds(groups: ToolbarGroup[], availableWidth: number): Set<string> {
  const collapsed = new Set<string>();
  if (availableWidth <= 0) return collapsed;

  let total =
    TOOLBAR_PADDING + HEADING_SELECT_WIDTH + groups.reduce((sum, group) => sum + expandedWidth(group), 0);

  for (const id of COLLAPSE_ORDER) {
    if (total <= availableWidth) break;
    const group = groups.find((candidate) => candidate.id === id);
    if (!group) continue;
    total -= expandedWidth(group) - (COLLAPSED_GROUP_WIDTH + DIVIDER_WIDTH);
    collapsed.add(id);
  }

  return collapsed;
}
