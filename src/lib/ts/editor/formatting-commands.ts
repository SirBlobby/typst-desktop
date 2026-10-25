import type { EditorView } from "@codemirror/view";
import { insertText, prefixLines, wrapSelection } from "#lib/ts/editor-actions";
import { insertList, setSectionLevel } from "#lib/ts/editor/latex-actions";
import type { DocumentLanguage } from "#lib/ts/document-language";

const HEADING_LEVELS = [1, 2, 3, 4, 5, 6];

export const formattingCommandIds = [
  "bold",
  "italic",
  "underline",
  "strikethrough",
  "link",
  "numberedList",
  "bulletedList",
  ...HEADING_LEVELS.map((level) => `heading${level}`),
];

function runTypstCommand(view: EditorView, id: string) {
  if (id === "bold") wrapSelection(view, "*", "*", "bold");
  else if (id === "italic") wrapSelection(view, "_", "_", "italic");
  else if (id === "underline") wrapSelection(view, "#underline[", "]", "underlined");
  else if (id === "strikethrough") wrapSelection(view, "#strike[", "]", "struck through");
  else if (id === "link") insertText(view, '#link("https://")[text]');
  else if (id === "numberedList") prefixLines(view, "+ ", "Numbered item");
  else if (id === "bulletedList") prefixLines(view, "- ", "List item");
  else if (id.startsWith("heading")) prefixLines(view, "=".repeat(Number(id.slice(7))) + " ", "Heading");
}

function runLatexCommand(view: EditorView, id: string) {
  if (id === "bold") wrapSelection(view, "\\textbf{", "}", "bold");
  else if (id === "italic") wrapSelection(view, "\\textit{", "}", "italic");
  else if (id === "underline") wrapSelection(view, "\\underline{", "}", "underlined");
  else if (id === "strikethrough") wrapSelection(view, "\\sout{", "}", "struck through");
  else if (id === "link") wrapSelection(view, "\\href{https://}{", "}", "text");
  else if (id === "numberedList") insertList(view, "enumerate");
  else if (id === "bulletedList") insertList(view, "itemize");
  else if (id.startsWith("heading")) setSectionLevel(view, Math.min(Number(id.slice(7)), 5));
}

export function runFormattingCommand(view: EditorView | null, id: string, language: DocumentLanguage) {
  if (!view) return;
  if (language === "latex") runLatexCommand(view, id);
  else runTypstCommand(view, id);
}
