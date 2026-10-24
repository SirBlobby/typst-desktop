import type { EditorView } from "@codemirror/view";
import { insertText, wrapSelection } from "$lib/ts/editor-actions";
import { openFind, openReplace } from "$lib/ts/editor/insert";
import { buildLatexTable, insertList, wrapEnvironment } from "$lib/ts/editor/latex-actions";
import type { ToolbarGroup } from "$lib/ts/editor/toolbar-groups";

export function buildLatexToolbarGroups(view: EditorView | null, onassets: () => void): ToolbarGroup[] {
  return [
    {
      id: "format",
      label: "Formatting",
      icon: "ph:text-aa",
      items: [
        { icon: "ph:text-b", label: "Bold", run: () => wrapSelection(view, "\\textbf{", "}", "bold") },
        { icon: "ph:text-italic", label: "Italic", run: () => wrapSelection(view, "\\textit{", "}", "italic") },
        { icon: "ph:text-underline", label: "Underline", run: () => wrapSelection(view, "\\underline{", "}", "underlined") },
        { icon: "ph:code", label: "Monospace", run: () => wrapSelection(view, "\\texttt{", "}", "code") },
        { icon: "ph:quotes", label: "Quote", run: () => wrapEnvironment(view, "quote", "Quote") },
      ],
    },
    {
      id: "math",
      label: "Math",
      icon: "ph:sigma",
      items: [
        { icon: "ph:sigma", label: "Inline math", run: () => wrapSelection(view, "$", "$", "x = y") },
        { icon: "ph:function", label: "Display math", run: () => wrapSelection(view, "\\[\n  ", "\n\\]", "x = y") },
        { icon: "ph:hash", label: "Numbered equation", run: () => wrapEnvironment(view, "equation", "x = y") },
      ],
    },
    {
      id: "lists",
      label: "Lists",
      icon: "ph:list-bullets",
      items: [
        { icon: "ph:list-bullets", label: "Bullet list", run: () => insertList(view, "itemize") },
        { icon: "ph:list-numbers", label: "Numbered list", run: () => insertList(view, "enumerate") },
      ],
    },
    {
      id: "insert",
      label: "Insert",
      icon: "ph:plus-square",
      items: [
        { icon: "ph:link", label: "Link", run: () => wrapSelection(view, "\\href{https://}{", "}", "text") },
        {
          icon: "ph:image-square",
          label: "Figure",
          run: () =>
            insertText(
              view,
              "\\begin{figure}[h]\n  \\centering\n  \\includegraphics[width=0.8\\linewidth]{file.png}\n  \\caption{Caption}\n  \\label{fig:label}\n\\end{figure}",
            ),
        },
        { icon: "ph:table", label: "Table", run: () => insertText(view, buildLatexTable(3, 3, true)) },
        { icon: "ph:images", label: "Images and fonts", run: onassets },
      ],
      menu: {
        icon: "ph:plus-square",
        label: "Insert",
        items: [
          { icon: "ph:note", label: "Footnote", run: () => wrapSelection(view, "\\footnote{", "}", "Footnote text") },
          { icon: "ph:tag", label: "Label", run: () => wrapSelection(view, "\\label{", "}", "sec:label") },
          { icon: "ph:arrow-bend-down-right", label: "Reference", run: () => wrapSelection(view, "\\ref{", "}", "sec:label") },
          { icon: "ph:book-open", label: "Citation", run: () => wrapSelection(view, "\\cite{", "}", "key") },
          { icon: "ph:code-block", label: "Verbatim block", run: () => wrapEnvironment(view, "verbatim", "code") },
          { icon: "ph:list-dashes", label: "Table of contents", run: () => insertText(view, "\\tableofcontents\n") },
          { icon: "ph:file-dashed", label: "Page break", run: () => insertText(view, "\\newpage\n") },
        ],
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
  ];
}
