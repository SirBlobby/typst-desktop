import { EditorView } from "@codemirror/view";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags as t } from "@lezer/highlight";
import type { ColorTheme } from "$lib/ts/state.svelte";

interface ThemeColors {
  background: string;
  surface: string;
  text: string;
  selection: string;
  activeLine: string;
  cursor: string;
  border: string;
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
      background: "#eff1f5", surface: "#e6e9ef", text: "#4c4f69", selection: "#ccd0da",
      activeLine: "#e6e9ef", cursor: "#dc8a78", border: "#ccd0da", keyword: "#8839ef",
      string: "#40a02b", number: "#fe640b", comment: "#8c8fa1", variable: "#4c4f69",
      function: "#1e66f5", heading: "#1e66f5",
    },
    dark: {
      background: "#1e1e2e", surface: "#181825", text: "#cdd6f4", selection: "#45475a",
      activeLine: "#313244", cursor: "#f5e0dc", border: "#313244", keyword: "#cba6f7",
      string: "#a6e3a1", number: "#fab387", comment: "#6c7086", variable: "#cdd6f4",
      function: "#89b4fa", heading: "#89b4fa",
    },
  },
  cerberus: {
    light: {
      background: "#ffffff", surface: "#f8f8fa", text: "#18181b", selection: "#c7d2fe",
      activeLine: "#f5f5f8", cursor: "#4f46e5", border: "#e2e2e8", keyword: "#9333ea",
      string: "#0f766e", number: "#c2410c", comment: "#71717a", variable: "#18181b",
      function: "#4f46e5", heading: "#4f46e5",
    },
    dark: {
      background: "#171717", surface: "#121212", text: "#f5f5f5", selection: "#3a3a4a",
      activeLine: "#262626", cursor: "#f5f5f5", border: "#262626", keyword: "#e879f9",
      string: "#2dd4bf", number: "#fbbf24", comment: "#737373", variable: "#f5f5f5",
      function: "#818cf8", heading: "#818cf8",
    },
  },
  archlinux: {
    light: {
      background: "#ffffff", surface: "#f5f7f9", text: "#1f2328", selection: "#b6e3ff",
      activeLine: "#f5f7f9", cursor: "#0a6ea8", border: "#d5dde3", keyword: "#cf222e",
      string: "#0a3069", number: "#0550ae", comment: "#6e7781", variable: "#1f2328",
      function: "#8250df", heading: "#0a6ea8",
    },
    dark: {
      background: "#0d1117", surface: "#010409", text: "#c9d1d9", selection: "#264f78",
      activeLine: "#161b22", cursor: "#c9d1d9", border: "#21262d", keyword: "#ff7b72",
      string: "#a5d6ff", number: "#79c0ff", comment: "#6e7681", variable: "#c9d1d9",
      function: "#d2a8ff", heading: "#1793d1",
    },
  },
};

export function editorTheme(isDark: boolean, colorTheme: ColorTheme) {
  const colors = (palettes[colorTheme] ?? palettes.catppuccin)[isDark ? "dark" : "light"];

  const theme = EditorView.theme(
    {
      "&": {
        color: colors.text,
        backgroundColor: colors.background,
        height: "100%",
        fontSize: "13px",
      },
      ".cm-content": {
        caretColor: colors.cursor,
        padding: "12px 0",
      },
      ".cm-cursor, .cm-dropCursor": { borderLeftColor: colors.cursor },
      "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection":
        { backgroundColor: colors.selection },
      ".cm-activeLine": { backgroundColor: colors.activeLine },
      ".cm-gutters": {
        backgroundColor: colors.background,
        color: colors.comment,
        border: "none",
      },
      ".cm-activeLineGutter": { backgroundColor: colors.activeLine },
      "&.cm-focused .cm-matchingBracket": {
        backgroundColor: colors.selection,
        outline: `1px solid ${colors.border}`,
      },

      ".cm-tooltip": {
        backgroundColor: colors.surface,
        color: colors.text,
        border: `1px solid ${colors.border}`,
        borderRadius: "3px",
        maxWidth: "500px",
      },
      ".cm-tooltip-hover": { maxHeight: "300px", overflow: "auto" },
      ".cm-tooltip .cm-tooltip-arrow:before": {
        borderTopColor: colors.border,
        borderBottomColor: colors.border,
      },
      ".cm-tooltip .cm-tooltip-arrow:after": {
        borderTopColor: colors.surface,
        borderBottomColor: colors.surface,
      },

      ".cm-tooltip.cm-tooltip-autocomplete": {
        backgroundColor: colors.surface,
        border: `1px solid ${colors.border}`,
        padding: "4px",
      },
      ".cm-tooltip.cm-tooltip-autocomplete > ul": {
        fontFamily: "inherit",
        maxHeight: "16em",
      },
      ".cm-tooltip.cm-tooltip-autocomplete > ul > li": {
        color: colors.text,
        padding: "3px 8px",
        borderRadius: "2px",
        display: "flex",
        alignItems: "center",
        gap: "6px",
      },
      ".cm-tooltip.cm-tooltip-autocomplete > ul > li[aria-selected]": {
        backgroundColor: colors.cursor,
        color: "#ffffff",
      },
      ".cm-tooltip.cm-tooltip-autocomplete > ul > li[aria-selected] .cm-completionDetail":
        { color: "#ffffff" },
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
        backgroundColor: colors.surface,
        color: colors.text,
        border: `1px solid ${colors.border}`,
        borderRadius: "3px",
        padding: "6px 8px",
      },

      ".cm-panels": { backgroundColor: colors.surface, color: colors.text },
      ".cm-searchMatch": { backgroundColor: "#72a1ff59" },
      ".cm-selectionMatch": { backgroundColor: "#aafe661a" },
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
