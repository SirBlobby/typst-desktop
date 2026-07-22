import { EditorView } from "@codemirror/view";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags as t } from "@lezer/highlight";

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

const palette: Record<"light" | "dark", ThemeColors> = {
  light: {
    background: "#ffffff",
    surface: "#f6f7f9",
    text: "#14161a",
    selection: "#dbe6fe",
    activeLine: "#f6f7f9",
    cursor: "#3b6cf6",
    border: "#dfe2e7",
    keyword: "#7c3aed",
    string: "#0f766e",
    number: "#b45309",
    comment: "#6b7280",
    variable: "#14161a",
    function: "#2563eb",
    heading: "#1d4ed8",
  },
  dark: {
    background: "#16181d",
    surface: "#1d2026",
    text: "#eef0f4",
    selection: "#3f5a91",
    activeLine: "#1d2026",
    cursor: "#6b93ff",
    border: "#2f343d",
    keyword: "#c4a7f7",
    string: "#8ddba4",
    number: "#f0b37e",
    comment: "#7b8496",
    variable: "#eef0f4",
    function: "#7aa2ff",
    heading: "#8fb3ff",
  },
};

export function editorTheme(isDark: boolean) {
  const colors = palette[isDark ? "dark" : "light"];

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
        borderRadius: "6px",
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
        borderRadius: "4px",
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
        borderRadius: "6px",
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
