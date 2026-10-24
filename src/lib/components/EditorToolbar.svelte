<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { EditorView } from "@codemirror/view";
  import * as api from "$lib/ts/api";
  import { setTypstConfig } from "$lib/ts/editor-actions";
  import { currentHeadingLevel, setHeadingLevel } from "$lib/ts/editor/insert";
  import {
    allGroupItems,
    buildToolbarGroups,
    collapsedGroupIds,
    type ToolbarItem,
  } from "$lib/ts/editor/toolbar-groups";
  import { buildLatexToolbarGroups } from "$lib/ts/editor/latex-toolbar-groups";
  import { currentSectionLevel, latexHeadingOptions, setSectionLevel } from "$lib/ts/editor/latex-actions";
  import type { DocumentLanguage } from "$lib/ts/document-language";
  import { clampMenu } from "$lib/ts/menu-position";
  import { app } from "$lib/ts/state.svelte";

  interface Props {
    view: EditorView | null;
    language?: DocumentLanguage;
    disabled?: boolean;
    onassets: () => void;
    onpagesettings: () => void;
  }

  let { view, language = "typst", disabled = false, onassets, onpagesettings }: Props = $props();

  let fonts = $state<string[]>([]);
  let selectedFont = $state("");
  let headingLevel = $state(0);
  let toolbarWidth = $state(0);
  let openMenu = $state<{ id: string; top: number; left: number } | null>(null);

  const typstHeadingOptions = [
    { level: 0, label: "Normal text" },
    ...[1, 2, 3, 4, 5, 6].map((level) => ({ level, label: `Heading ${level}` })),
  ];

  const isLatex = $derived(language === "latex");
  const headingOptions = $derived(isLatex ? latexHeadingOptions : typstHeadingOptions);
  const groups = $derived(
    isLatex ? buildLatexToolbarGroups(view, onassets) : buildToolbarGroups(view, onassets, onpagesettings),
  );
  const collapsed = $derived(collapsedGroupIds(groups, toolbarWidth));

  const openMenuItems = $derived.by((): ToolbarItem[] => {
    if (!openMenu) return [];
    const [groupId, kind] = openMenu.id.split(":");
    const group = groups.find((candidate) => candidate.id === groupId);
    if (!group) return [];
    return kind === "menu" ? (group.menu?.items ?? []) : allGroupItems(group);
  });

  const showFontInMenu = $derived(openMenu?.id === "document:group");

  function syncHeadingLevel() {
    headingLevel = isLatex ? currentSectionLevel(view) : currentHeadingLevel(view);
  }

  function applyHeading(level: number) {
    if (!view) return;
    if (isLatex) setSectionLevel(view, level);
    else setHeadingLevel(view, level);
    headingLevel = level;
  }

  function applyFont(family: string) {
    selectedFont = family;
    if (family) setTypstConfig(view, "text", "font", `"${family}"`);
  }

  function toggleMenu(event: MouseEvent, id: string) {
    event.stopPropagation();
    if (openMenu?.id === id) {
      openMenu = null;
      return;
    }
    const bounds = (event.currentTarget as HTMLElement).getBoundingClientRect();
    openMenu = { id, top: bounds.bottom + 4, left: bounds.left };
  }

  function runItem(item: ToolbarItem) {
    openMenu = null;
    item.run();
  }

  $effect(() => {
    const path = app.target?.path;
    api
      .listFontFamilies(path)
      .then((families) => (fonts = families))
      .catch(() => (fonts = []));
  });
</script>

<svelte:window onclick={() => (openMenu = null)} onkeyup={syncHeadingLevel} onmouseup={syncHeadingLevel} />

{#snippet fontSelect(extraClass: string)}
  <select
    class="field field-sm h-7 py-0 shadow-none {extraClass}"
    aria-label="Document font"
    {disabled}
    value={selectedFont}
    onchange={(event) => applyFont(event.currentTarget.value)}
  >
    <option value="">Font</option>
    {#each fonts as family}
      <option value={family}>{family}</option>
    {/each}
  </select>
{/snippet}

{#snippet dropdownButton(id: string, icon: string, label: string)}
  <button
    class="icon-btn w-auto gap-0.5 px-1.5 {openMenu?.id === id ? 'icon-btn-active' : ''}"
    title={label}
    aria-label={label}
    aria-haspopup="menu"
    aria-expanded={openMenu?.id === id}
    {disabled}
    onclick={(event) => toggleMenu(event, id)}
  >
    <Icon {icon} class="text-base" />
    <Icon icon="ph:caret-down" class="text-[10px]" />
  </button>
{/snippet}

<div class="bar gap-0.5 overflow-hidden px-2" bind:clientWidth={toolbarWidth}>
  <select
    class="field field-sm h-7 w-auto shrink-0 py-0 shadow-none"
    aria-label="Text style"
    {disabled}
    value={headingLevel}
    onchange={(event) => applyHeading(Number(event.currentTarget.value))}
  >
    {#each headingOptions as option (option.level)}
      <option value={option.level}>{option.label}</option>
    {/each}
  </select>

  {#each groups as group (group.id)}
    <div class="divider-v"></div>
    {#if collapsed.has(group.id)}
      {@render dropdownButton(`${group.id}:group`, group.icon, group.label)}
    {:else}
      {#each group.items as item (item.label)}
        <button class="icon-btn" title={item.label} aria-label={item.label} {disabled} onclick={item.run}>
          <Icon icon={item.icon} class="text-base" />
        </button>
      {/each}
      {#if group.menu}
        {@render dropdownButton(`${group.id}:menu`, group.menu.icon, group.menu.label)}
      {/if}
      {#if group.id === "document"}
        {@render fontSelect("w-auto max-w-36 shrink-0")}
      {/if}
    {/if}
  {/each}
</div>

{#if openMenu}
  <div
    use:clampMenu
    role="menu"
    tabindex="-1"
    class="menu fixed z-50 w-56"
    style="top: {openMenu.top}px; left: {openMenu.left}px;"
    onclick={(event) => event.stopPropagation()}
    onkeydown={(event) => event.key === "Escape" && (openMenu = null)}
  >
    {#each openMenuItems as item (item.label)}
      <button role="menuitem" class="menu-item" onclick={() => runItem(item)}>
        <Icon icon={item.icon} class="text-sm text-[var(--color-ink-muted)]" />
        {item.label}
      </button>
    {/each}
    {#if showFontInMenu}
      <div class="menu-separator"></div>
      <div class="px-1.5 pb-1">{@render fontSelect("w-full")}</div>
    {/if}
  </div>
{/if}
