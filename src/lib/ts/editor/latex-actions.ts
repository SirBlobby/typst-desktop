import type { EditorView } from "@codemirror/view";
import { wrapSelection } from "$lib/ts/editor-actions";

export const latexSectionCommands = ["section", "subsection", "subsubsection", "paragraph", "subparagraph"];

export const latexHeadingOptions = [
  { level: 0, label: "Normal text" },
  { level: 1, label: "Section" },
  { level: 2, label: "Subsection" },
  { level: 3, label: "Subsubsection" },
  { level: 4, label: "Paragraph" },
  { level: 5, label: "Subparagraph" },
];

const SECTION_PATTERN = /^(\s*)\\(section|subsection|subsubsection|paragraph|subparagraph)\*?\{(.*)\}\s*$/;

export function currentSectionLevel(view: EditorView | null) {
  if (!view) return 0;
  const line = view.state.doc.lineAt(view.state.selection.main.head);
  const match = line.text.match(SECTION_PATTERN);
  return match ? latexSectionCommands.indexOf(match[2]) + 1 : 0;
}

export function setSectionLevel(view: EditorView, level: number) {
  const line = view.state.doc.lineAt(view.state.selection.main.head);
  const match = line.text.match(SECTION_PATTERN);
  const indent = match ? match[1] : (line.text.match(/^\s*/)?.[0] ?? "");
  const title = match ? match[3] : line.text.trim();
  const command = latexSectionCommands[level - 1];
  const replacement = command ? `${indent}\\${command}{${title || "Title"}}` : `${indent}${title}`;

  view.dispatch({
    changes: { from: line.from, to: line.to, insert: replacement },
    selection: { anchor: line.from + replacement.length },
  });
  view.focus();
}

export function wrapEnvironment(view: EditorView | null, environment: string, placeholder: string) {
  wrapSelection(view, `\\begin{${environment}}\n  `, `\n\\end{${environment}}`, placeholder);
}

export function insertList(view: EditorView | null, environment: "itemize" | "enumerate") {
  if (!view) return;
  const selection = view.state.selection.main;
  const startLine = view.state.doc.lineAt(selection.from);
  const endLine = view.state.doc.lineAt(selection.to);
  const selectedLines = selection.empty
    ? []
    : view.state.doc.sliceString(startLine.from, endLine.to).split("\n").filter((text) => text.trim() !== "");

  const items = (selectedLines.length > 0 ? selectedLines : ["List item"])
    .map((text) => `  \\item ${text.trim().replace(/^\\item\s*/, "")}`)
    .join("\n");
  const block = `\\begin{${environment}}\n${items}\n\\end{${environment}}`;
  const from = selection.empty ? selection.from : startLine.from;
  const to = selection.empty ? selection.to : endLine.to;

  view.dispatch({
    changes: { from, to, insert: block },
    selection: { anchor: from + block.length },
  });
  view.focus();
}

export function buildLatexTable(rowCount: number, columnCount: number, withHeader: boolean) {
  const row = (label: string, rowNumber: number) =>
    "    " + Array.from({ length: columnCount }, (_, column) => `${label} ${rowNumber}.${column + 1}`).join(" & ") + " \\\\";
  const lines = [
    "\\begin{table}[h]",
    "  \\centering",
    `  \\begin{tabular}{${"l".repeat(columnCount)}}`,
    "    \\hline",
  ];
  if (withHeader) {
    lines.push("    " + Array.from({ length: columnCount }, (_, column) => `\\textbf{Header ${column + 1}}`).join(" & ") + " \\\\");
    lines.push("    \\hline");
  }
  for (let rowNumber = 1; rowNumber <= rowCount; rowNumber++) {
    lines.push(row("Cell", rowNumber));
  }
  lines.push("    \\hline", "  \\end{tabular}", "  \\caption{Caption}", "\\end{table}");
  return lines.join("\n");
}
