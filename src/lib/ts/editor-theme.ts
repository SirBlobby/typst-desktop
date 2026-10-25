import { EditorView } from "@codemirror/view";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags as t } from "@lezer/highlight";
import type { ColorTheme } from "#lib/ts/state.svelte";

interface ThemeColors {
  text: string;
  cursor: string;
  keyword: string;
  string: string;
  number: string;
  comment: string;
  variable: string;
  function: string;
  heading: string;
}

type Mode = "light" | "dark";

const palettes: Record<ColorTheme, Record<Mode, ThemeColors>> = {
  catppuccin: {
    light: {
      text: "#4c4f69", cursor: "#dc8a78", keyword: "#8839ef",
      string: "#40a02b", number: "#fe640b", comment: "#8c8fa1", variable: "#4c4f69",
      function: "#1e66f5", heading: "#1e66f5",
    },
    dark: {
      text: "#cdd6f4", cursor: "#f5e0dc", keyword: "#cba6f7",
      string: "#a6e3a1", number: "#fab387", comment: "#6c7086", variable: "#cdd6f4",
      function: "#89b4fa", heading: "#89b4fa",
    },
  },
  cerberus: {
    light: {
      text: "#18181b", cursor: "#4f46e5", keyword: "#9333ea",
      string: "#0f766e", number: "#c2410c", comment: "#71717a", variable: "#18181b",
      function: "#4f46e5", heading: "#4f46e5",
    },
    dark: {
      text: "#f5f5f5", cursor: "#f5f5f5", keyword: "#e879f9",
      string: "#2dd4bf", number: "#fbbf24", comment: "#737373", variable: "#f5f5f5",
      function: "#818cf8", heading: "#818cf8",
    },
  },
  archlinux: {
    light: {
      text: "#1f2328", cursor: "#0a6ea8", keyword: "#cf222e",
      string: "#0a3069", number: "#0550ae", comment: "#6e7781", variable: "#1f2328",
      function: "#8250df", heading: "#0a6ea8",
    },
    dark: {
      text: "#c9d1d9", cursor: "#c9d1d9", keyword: "#ff7b72",
      string: "#a5d6ff", number: "#79c0ff", comment: "#6e7681", variable: "#c9d1d9",
      function: "#d2a8ff", heading: "#1793d1",
    },
  },
};

const surfaces = {
  background: "var(--color-editor)",
  popover: "var(--color-surface-raised)",
  border: "var(--color-line)",
  activeLine: "color-mix(in srgb, var(--color-ink) 4%, transparent)",
  selection: "color-mix(in srgb, var(--color-accent) 32%, transparent)",
  selectionUnfocused: "color-mix(in srgb, var(--color-ink) 14%, transparent)",
  selectionMatch: "color-mix(in srgb, var(--color-accent) 14%, transparent)",
  highlighted: "var(--color-accent-soft)",
};

export function editorTheme(isDark: boolean, colorTheme: ColorTheme) {
  const colors = (palettes[colorTheme] ?? palettes.catppuccin)[isDark ? "dark" : "light"];

  const theme = EditorView.theme(
    {
      "&": {
        color: colors.text,
        backgroundColor: surfaces.background,
        height: "100%",
      },
      ".cm-content": {
        caretColor: colors.cursor,
        padding: "12px 0",
      },
      ".cm-cursor, .cm-dropCursor": { borderLeftColor: colors.cursor },
      ".cm-selectionBackground": { backgroundColor: surfaces.selectionUnfocused },
      "&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground": {
        backgroundColor: surfaces.selection,
      },
      ".cm-activeLine": { backgroundColor: surfaces.activeLine },
      ".cm-gutters": {
        backgroundColor: surfaces.background,
        color: colors.comment,
        border: "none",
      },
      ".cm-activeLineGutter": { backgroundColor: surfaces.activeLine, color: colors.text },
      "&.cm-focused .cm-matchingBracket": {
        backgroundColor: surfaces.selectionMatch,
        outline: `1px solid ${surfaces.border}`,
      },

      ".cm-tooltip": {
        backgroundColor: surfaces.popover,
        color: colors.text,
        border: `1px solid ${surfaces.border}`,
        borderRadius: "8px",
        boxShadow: "var(--shadow-lg)",
        maxWidth: "500px",
      },
      ".cm-tooltip-hover": { maxHeight: "300px", overflow: "auto" },
      ".cm-tooltip .cm-tooltip-arrow:before": {
        borderTopColor: surfaces.border,
        borderBottomColor: surfaces.border,
      },
      ".cm-tooltip .cm-tooltip-arrow:after": {
        borderTopColor: surfaces.popover,
        borderBottomColor: surfaces.popover,
      },

      ".cm-tooltip.cm-tooltip-autocomplete": {
        backgroundColor: surfaces.popover,
        border: `1px solid ${surfaces.border}`,
        padding: "4px",
      },
      ".cm-tooltip.cm-tooltip-autocomplete > ul": {
        fontFamily: "inherit",
        maxHeight: "16em",
      },
      ".cm-tooltip.cm-tooltip-autocomplete > ul > li": {
        color: colors.text,
        padding: "3px 8px",
        borderRadius: "4px",
        display: "flex",
        alignItems: "center",
        gap: "6px",
      },
      ".cm-tooltip.cm-tooltip-autocomplete > ul > li[aria-selected]": {
        backgroundColor: surfaces.highlighted,
        color: colors.text,
      },
      ".cm-tooltip.cm-tooltip-autocomplete > ul > li[aria-selected] .cm-completionDetail":
        { color: colors.text },
      ".cm-completionLabel": { color: "inherit" },
      ".cm-completionMatchedText": {
        textDecoration: "none",
        fontWeight: "600",
        color: "inherit",
      },
      ".cm-completionDetail": {
        color: colors.comment,
        fontStyle: "normal",
        marginLeft: "auto",
        fontSize: "0.85em",
      },
      ".cm-completionIcon": {
        color: colors.comment,
        opacity: "1",
        width: "1.1em",
      },
      ".cm-completionInfo": {
        backgroundColor: surfaces.popover,
        color: colors.text,
        border: `1px solid ${surfaces.border}`,
        borderRadius: "8px",
        padding: "6px 8px",
      },

      ".cm-selectionMatch": { backgroundColor: surfaces.selectionMatch },
    },
    { dark: isDark },
  );

  const highlightStyle = HighlightStyle.define([
    { tag: t.keyword, color: colors.keyword },
    {
      tag: [t.name, t.deleted, t.character, t.propertyName, t.macroName],
      color: colors.variable,
    },
    { tag: [t.function(t.variableName), t.labelName], color: colors.function },
    {
      tag: [t.color, t.constant(t.name), t.standard(t.name)],
      color: colors.function,
    },
    { tag: [t.definition(t.name), t.separator], color: colors.variable },
    {
      tag: [
        t.typeName,
        t.className,
        t.number,
        t.changed,
        t.annotation,
        t.modifier,
        t.self,
        t.namespace,
      ],
      color: colors.number,
    },
    {
      tag: [
        t.operator,
        t.operatorKeyword,
        t.url,
        t.escape,
        t.regexp,
        t.special(t.string),
      ],
      color: colors.keyword,
    },
    { tag: [t.meta, t.comment], color: colors.comment, fontStyle: "italic" },
    { tag: t.strong, fontWeight: "bold" },
    { tag: t.emphasis, fontStyle: "italic" },
    { tag: t.strikethrough, textDecoration: "line-through" },
    { tag: t.link, color: colors.function, textDecoration: "underline" },
    { tag: t.heading, fontWeight: "bold", color: colors.heading },
    { tag: [t.atom, t.bool, t.special(t.variableName)], color: colors.number },
    {
      tag: [t.processingInstruction, t.string, t.inserted],
      color: colors.string,
    },
    { tag: t.invalid, color: "#ff5c57" },
  ]);

  return [theme, syntaxHighlighting(highlightStyle, { fallback: true })];
}
