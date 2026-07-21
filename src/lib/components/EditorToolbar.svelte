<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { EditorView } from "@codemirror/view";
  import * as api from "$lib/ts/api";
  import {
    insertText,
    prefixLines,
    setTypstConfig,
    wrapSelection,
  } from "$lib/ts/editor-actions";
  import { app } from "$lib/ts/state.svelte";

  interface Props {
    view: EditorView | null;
    disabled?: boolean;
    onassets: () => void;
    onpagesettings: () => void;
  }

  let { view, disabled = false, onassets, onpagesettings }: Props = $props();

  let fonts = $state<string[]>([]);
  let selectedFont = $state("");

  $effect(() => {
    const path = app.target?.path;
    api
      .listFontFamilies(path)
      .then((families) => (fonts = families))
      .catch(() => (fonts = []));
  });
</script>

{#snippet action(
  icon: string,
  label: string,
  run: () => void,
  size = "text-sm",
)}
  <button
    class="rounded p-1 text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] disabled:opacity-40 disabled:hover:bg-transparent"
    title={label}
    aria-label={label}
    {disabled}
    onclick={run}
  >
    <Icon {icon} class={size} />
  </button>
{/snippet}

{#snippet divider()}
  <div class="mx-0.5 h-3.5 w-px shrink-0 bg-[var(--color-line)]"></div>
{/snippet}

<div
  class="scroll-thin flex shrink-0 items-center gap-px overflow-x-auto border-b border-[var(--color-line)] bg-[var(--color-surface)] px-1.5 py-0.5"
>
  {@render action("ph:text-h", "Heading", () => prefixLines(view, "= ", "Heading"))}
  {@render action("ph:text-b", "Bold", () => wrapSelection(view, "*", "*", "bold"))}
  {@render action("ph:text-italic", "Italic", () =>
    wrapSelection(view, "_", "_", "italic"),
  )}
  {@render action("ph:code", "Raw", () => wrapSelection(view, "`", "`", "code"))}

  {@render divider()}

  {@render action("ph:sigma", "Inline math", () =>
    wrapSelection(view, "$", "$", "x = y"),
  )}
  {@render action("ph:function", "Block math", () =>
    wrapSelection(view, "$ \n  ", "\n$", "x = y"),
  )}

  {@render divider()}

  {@render action("ph:list-bullets", "Bullet list", () =>
    prefixLines(view, "- ", "List item"),
  )}
  {@render action("ph:list-numbers", "Numbered list", () =>
    prefixLines(view, "+ ", "Numbered item"),
  )}

  {@render divider()}

  {@render action("ph:link", "Link", () =>
    insertText(view, '#link("https://")[text]'),
  )}
  {@render action("ph:table", "Table", () =>
    insertText(view, "#table(\n  columns: 2,\n  [a], [b],\n)"),
  )}
  {@render action("ph:image-square", "Figure", () =>
    insertText(view, '#figure(\n  image("file.png"),\n  caption: [Caption],\n)'),
  )}
  {@render action("ph:images", "Images and fonts", onassets)}

  {@render divider()}

  <select
    class="max-w-24 rounded border border-[var(--color-line)] bg-[var(--color-surface)] px-1 py-0.5 text-xs text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none disabled:opacity-40"
    aria-label="Document font"
    {disabled}
    bind:value={selectedFont}
    onchange={(event) => {
      const family = event.currentTarget.value;
      if (family) setTypstConfig(view, "text", "font", `"${family}"`);
    }}
  >
    <option value="">Font</option>
    {#each fonts as family}
      <option value={family}>{family}</option>
    {/each}
  </select>

  {@render action("ph:file-text", "Page settings", onpagesettings)}

  <div class="flex-1"></div>
</div>
