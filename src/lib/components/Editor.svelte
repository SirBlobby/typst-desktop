<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {
    EditorView,
    keymap,
    lineNumbers,
    highlightActiveLine,
  } from "@codemirror/view";
  import { EditorState, Compartment, StateField } from "@codemirror/state";
  import { defaultKeymap, history, indentWithTab } from "@codemirror/commands";
  import {
    bracketMatching,
    indentOnInput,
    language,
    Language,
    StreamLanguage,
  } from "@codemirror/language";
  import { toml } from "@codemirror/legacy-modes/mode/toml";
  import {
    autocompletion,
    closeBrackets,
    closeBracketsKeymap,
  } from "@codemirror/autocomplete";
  import { lintGutter, setDiagnostics } from "@codemirror/lint";
  import { LSPClient, languageServerExtensions } from "@codemirror/lsp-client";
  import { yCollab } from "y-codemirror.next";
  import type { Awareness } from "y-protocols/awareness";
  import type * as Y from "yjs";

  import { typstCompletions } from "$lib/ts/completions";
  import { editorTheme } from "$lib/ts/editor-theme";
  import { LspBridge } from "$lib/ts/lsp";
  import { app } from "$lib/ts/state.svelte";
  import type { Diagnostic } from "$lib/ts/api";

  interface Props {
    content: string;
    filePath: string;
    targetPath: string;
    enableLsp?: boolean;
    diagnostics?: Diagnostic[];
    collab?: { text: Y.Text; awareness: Awareness } | null;
    onchange: (value: string) => void;
    onlspstatus?: (status: "off" | "starting" | "on" | "unavailable") => void;
    onready?: (view: EditorView | null) => void;
  }

  let {
    content,
    filePath,
    targetPath,
    enableLsp = true,
    diagnostics = [],
    collab = null,
    onchange,
    onlspstatus,
    onready,
  }: Props = $props();

  let host: HTMLDivElement;
  let view: EditorView | null = null;

  const languageSlot = new Compartment();
  const lspSlot = new Compartment();
  const themeSlot = new Compartment();
  const collabSlot = new Compartment();

  const bridge = new LspBridge();
  let client: LSPClient | null = null;

  const isToml = $derived(filePath.toLowerCase().endsWith(".toml"));

  function disableTypstLanguage() {
    queueMicrotask(() => {
      view?.dispatch({ effects: languageSlot.reconfigure([]) });
    });
  }

  function resilientSync(parser: any) {
    return StateField.define<null>({
      create: () => null,
      update(_value, transaction) {
        if (
          transaction.startState.facet(language) !==
          transaction.state.facet(language)
        ) {
          try {
            parser.clearParser();
          } catch {
            disableTypstLanguage();
          }
          return null;
        }

        if (!transaction.docChanged) return null;

        try {
          transaction.changes.iterChanges((fromA, toA, _fromB, _toB, inserted) => {
            const edits = parser.parser?.edit(fromA, toA, inserted.toString());
            if (!edits || edits.full_update) {
              parser.clearTree();
              return;
            }
            for (const edit of edits.edits) parser.applyTreeEdit(edit);
          });
        } catch {
          try {
            parser.clearParser();
          } catch {
            // The wasm parser instance panicked and is no longer usable.
            // Drop syntax highlighting rather than crash on every keystroke.
            disableTypstLanguage();
          }
        }

        return null;
      },
    });
  }

  async function loadTypstLanguage() {
    const { typst, TypstParser, typstHighlight } = await import(
      "codemirror-lang-typst"
    );
    const support = typst();
    const parser = new (TypstParser as any)(typstHighlight);
    return new Language(
      support.language.data,
      parser,
      [resilientSync(parser)],
      "typst",
    );
  }

  async function connectLsp() {
    if (!enableLsp || isToml || !view) return;

    onlspstatus?.("starting");
    try {
      const handle = await bridge.start(targetPath, () => {
        onlspstatus?.("off");
      });

      client = new LSPClient({
        rootUri: handle.root_uri,
        timeout: 10000,
        extensions: languageServerExtensions(),
      }).connect(bridge.transport);

      const documentUri = `${handle.root_uri}/${filePath}`;
      view.dispatch({
        effects: lspSlot.reconfigure(client.plugin(documentUri, "typst")),
      });
      onlspstatus?.("on");
    } catch {
      onlspstatus?.("unavailable");
    }
  }

  onMount(() => {
    view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: content,
        extensions: [
          lineNumbers(),
          lintGutter(),
          history(),
          bracketMatching(),
          closeBrackets(),
          indentOnInput(),
          highlightActiveLine(),
          languageSlot.of(isToml ? StreamLanguage.define(toml) : []),
          lspSlot.of([]),
          themeSlot.of(editorTheme(app.theme === "dark")),
          collabSlot.of([]),
          ...(isToml
            ? []
            : [autocompletion({ override: [typstCompletions] })]),
          EditorView.lineWrapping,
          keymap.of([
            ...closeBracketsKeymap,
            ...defaultKeymap,
            indentWithTab,
          ]),
          EditorView.updateListener.of((update) => {
            if (update.docChanged) onchange(update.state.doc.toString());
          }),
        ],
      }),
    });

    if (!isToml) {
      loadTypstLanguage()
        .then((language) => {
          view?.dispatch({ effects: languageSlot.reconfigure(language) });
        })
        .catch(() => {});
    }

    onready?.(view);

    connectLsp();
  });

  onDestroy(() => {
    onready?.(null);
    bridge.stop();
    view?.destroy();
  });

  function laterDispatch(build: (current: EditorView) => void) {
    queueMicrotask(() => {
      if (!view) return;
      build(view);
    });
  }

  let boundCollab: { text: Y.Text; awareness: Awareness } | null = null;

  $effect(() => {
    const next = collab;
    if (next === boundCollab) return;

    laterDispatch((current) => {
      if (next) {
        current.dispatch({
          changes: {
            from: 0,
            to: current.state.doc.length,
            insert: next.text.toString(),
          },
          effects: collabSlot.reconfigure([
            yCollab(next.text, next.awareness),
          ]),
        });
      } else {
        current.dispatch({ effects: collabSlot.reconfigure([]) });
      }
      boundCollab = next;
    });
  });

  $effect(() => {
    const next = content;
    if (!view || collab) return;
    if (view.state.doc.toString() === next) return;

    laterDispatch((current) => {
      if (current.state.doc.toString() === next) return;
      current.dispatch({
        changes: { from: 0, to: current.state.doc.length, insert: next },
      });
    });
  });

  $effect(() => {
    const isDark = app.theme === "dark";
    laterDispatch((current) => {
      current.dispatch({ effects: themeSlot.reconfigure(editorTheme(isDark)) });
    });
  });

  $effect(() => {
    const entries = diagnostics;
    if (!view) return;

    laterDispatch((current) => {
      const doc = current.state.doc;
      const marks = entries
        .filter((entry) => entry.line !== null)
        .map((entry) => {
          const line = doc.line(
            Math.min(Math.max(entry.line ?? 1, 1), doc.lines),
          );
          const from = Math.min(
            line.from + Math.max((entry.column ?? 1) - 1, 0),
            line.to,
          );
          return {
            from,
            to: line.to,
            severity: entry.severity.includes("warn")
              ? ("warning" as const)
              : ("error" as const),
            message: entry.message,
          };
        });

      current.dispatch(setDiagnostics(current.state, marks));
    });
  });
</script>

<div class="h-full overflow-hidden bg-[var(--color-surface)]" bind:this={host}></div>
