import type { EditorView } from "@codemirror/view";
import { undo, redo } from "@codemirror/commands";

export function insertText(view: EditorView | null, text: string) {
  if (!view) return;
  const { from, to } = view.state.selection.main;
  view.dispatch({
    changes: { from, to, insert: text },
    selection: { anchor: from + text.length },
  });
  view.focus();
}

export function wrapSelection(
  view: EditorView | null,
  prefix: string,
  suffix: string,
  placeholder = "",
) {
  if (!view) return;
  const { state } = view;
  const selection = state.selection.main;
  const selected = state.doc.sliceString(selection.from, selection.to);

  const innerWrapped =
    selected.length >= prefix.length + suffix.length &&
    selected.startsWith(prefix) &&
    selected.endsWith(suffix);

  const before = state.doc.sliceString(
    Math.max(0, selection.from - prefix.length),
    selection.from,
  );
  const after = state.doc.sliceString(
    selection.to,
    Math.min(state.doc.length, selection.to + suffix.length),
  );
  const outerWrapped = before === prefix && after === suffix;

  if (innerWrapped) {
    const inner = selected.slice(prefix.length, selected.length - suffix.length);
    view.dispatch({
      changes: { from: selection.from, to: selection.to, insert: inner },
      selection: { anchor: selection.from, head: selection.from + inner.length },
    });
  } else if (outerWrapped) {
    const from = selection.from - prefix.length;
    const to = selection.to + suffix.length;
    view.dispatch({
      changes: { from, to, insert: selected },
      selection: { anchor: from, head: from + selected.length },
    });
  } else {
    const body = selected || placeholder;
    view.dispatch({
      changes: { from: selection.from, to: selection.to, insert: prefix + body + suffix },
      selection: {
        anchor: selection.from + prefix.length,
        head: selection.from + prefix.length + body.length,
      },
    });
  }
  view.focus();
}

export function prefixLines(
  view: EditorView | null,
  prefix: string,
  placeholder = "",
) {
  if (!view) return;
  const selection = view.state.selection.main;
  const startLine = view.state.doc.lineAt(selection.from);
  const endLine = view.state.doc.lineAt(selection.to);

  if (selection.empty && startLine.text.trim() === "") {
    insertText(view, prefix + placeholder);
    return;
  }

  const changes = [];
  for (let number = startLine.number; number <= endLine.number; number += 1) {
    const line = view.state.doc.line(number);
    if (line.text.startsWith(prefix)) continue;
    changes.push({ from: line.from, insert: prefix });
  }

  view.dispatch({ changes });
  view.focus();
}

export function undoEdit(view: EditorView | null) {
  if (!view) return;
  undo(view);
  view.focus();
}

export function redoEdit(view: EditorView | null) {
  if (!view) return;
  redo(view);
  view.focus();
}

export function setTypstConfig(
  view: EditorView | null,
  setting: string,
  property: string,
  value: string,
) {
  if (!view) return;

  const content = view.state.doc.toString();
  const rule = new RegExp(`^#set\\s+${setting}\\s*\\(([^)]*)\\)`, "m");
  const match = content.match(rule);

  if (!match || match.index === undefined) {
    view.dispatch({
      changes: { from: 0, insert: `#set ${setting}(${property}: ${value})\n` },
    });
    view.focus();
    return;
  }

  const existing = match[1];
  const property_rule = new RegExp(
    `${property}\\s*:\\s*(?:\\([^)]*\\)|"[^"]*"|[^,)]+)`,
  );

  const next = property_rule.test(existing)
    ? existing.replace(property_rule, `${property}: ${value}`)
    : existing.trim()
      ? `${existing}, ${property}: ${value}`
      : `${property}: ${value}`;

  view.dispatch({
    changes: {
      from: match.index,
      to: match.index + match[0].length,
      insert: `#set ${setting}(${next})`,
    },
  });
  view.focus();
}
