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
  import {
    currentHeadingLevel,
    insertQuote,
    openFind,
    openReplace,
    setHeadingLevel,
  } from "$lib/ts/editor/insert";
  import { insertDialogStore, insertOptions } from "$lib/ts/editor/insert-dialog";
  import { clampMenu } from "$lib/ts/menu-position";
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
  let headingLevel = $state(0);
  let insertMenuPosition = $state<{ top: number; left: number } | null>(null);

  const headingOptions = [
    { level: 0, label: "Normal text" },
    ...[1, 2, 3, 4, 5, 6].map((level) => ({ level, label: `Heading ${level}` })),
  ];

  function syncHeadingLevel() {
    headingLevel = currentHeadingLevel(view);
  }

  function applyHeading(level: number) {
    if (!view) return;
    setHeadingLevel(view, level);
    headingLevel = level;
  }

  function toggleInsertMenu(event: MouseEvent) {
    event.stopPropagation();
    if (insertMenuPosition) {
      insertMenuPosition = null;
      return;
    }
    const bounds = (event.currentTarget as HTMLElement).getBoundingClientRect();
    insertMenuPosition = { top: bounds.bottom + 4, left: bounds.left };
  }

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
  size = "text-base",
)}
  <button
    class="rounded-md p-1.5 text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] disabled:opacity-40 disabled:hover:bg-transparent"
    title={label}
    aria-label={label}
    {disabled}
    onclick={run}
  >
    <Icon {icon} class={size} />
  </button>
{/snippet}

<svelte:window onclick={() => (insertMenuPosition = null)} onkeyup={syncHeadingLevel} onmouseup={syncHeadingLevel} />

{#snippet divider()}
  <div class="mx-1.5 h-5 w-px shrink-0 bg-[var(--color-line)]"></div>
{/snippet}

<div
  class="scroll-thin flex shrink-0 items-center gap-1 overflow-x-auto border-b border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-1.5"
>
  <select
    class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-2 py-1 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none disabled:opacity-40"
    aria-label="Text style"
    {disabled}
    value={headingLevel}
    onchange={(event) => applyHeading(Number(event.currentTarget.value))}
  >
    {#each headingOptions as option (option.level)}
      <option value={option.level}>{option.label}</option>
    {/each}
  </select>

  {@render divider()}

  {@render action("ph:text-b", "Bold", () => wrapSelection(view, "*", "*", "bold"))}
  {@render action("ph:text-italic", "Italic", () =>
    wrapSelection(view, "_", "_", "italic"),
  )}
  {@render action("ph:code", "Raw", () => wrapSelection(view, "`", "`", "code"))}
  {@render action("ph:quotes", "Quote", () => view && insertQuote(view))}

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
  {@render action("ph:image-square", "Figure", () =>
    insertText(view, '#figure(\n  image("file.png"),\n  caption: [Caption],\n)'),
  )}
  <button
    class="flex items-center gap-0.5 rounded-md p-1.5 text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] disabled:opacity-40 disabled:hover:bg-transparent"
    title="Insert"
    aria-label="Insert"
    {disabled}
    onclick={toggleInsertMenu}
  >
    <Icon icon="ph:plus-square" class="text-base" />
    <Icon icon="ph:caret-down" class="text-xs" />
  </button>
  {@render action("ph:images", "Images and fonts", onassets)}

  {@render divider()}

  {@render action("ph:magnifying-glass", "Find", () => view && openFind(view))}
  {@render action("ph:swap", "Search and replace", () => view && openReplace(view))}

  {@render divider()}

  <select
    class="max-w-32 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-2 py-1 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none disabled:opacity-40"
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

{#if insertMenuPosition}
  <div
    use:clampMenu
    class="fixed z-50 w-52 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] py-1 shadow-xl"
    style="top: {insertMenuPosition.top}px; left: {insertMenuPosition.left}px;"
  >
    {#each insertOptions as option (option.dialog)}
      <button
        class="flex w-full items-center gap-2 px-3 py-2 text-left text-sm hover:bg-[var(--color-surface-sunken)]"
        onclick={() => {
          insertMenuPosition = null;
          insertDialogStore.set(option.dialog);
        }}
      >
        <Icon icon={option.icon} class="text-base text-[var(--color-ink-muted)]" />
        {option.label}
      </button>
    {/each}
  </div>
{/if}
