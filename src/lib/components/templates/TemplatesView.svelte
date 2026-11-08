<script lang="ts">
  import Icon from "@iconify/svelte";
  import TemplateBrowser from "./TemplateBrowser.svelte";
  import TemplateCreateModal from "./TemplateCreateModal.svelte";
  import { app } from "#lib/ts/state.svelte";
  import type { TemplateAuthor, UniverseTemplate } from "#lib/ts/templates";

  let {
    oncreate,
  }: {
    oncreate: (template: UniverseTemplate, title: string, authors: TemplateAuthor[]) => Promise<void>;
  } = $props();

  let selected = $state<UniverseTemplate | null>(null);

  function folderLabel(currentDir: string) {
    const shortcut = app.shortcuts.find(
      (entry) => currentDir === entry.browse_path || currentDir.startsWith(`${entry.browse_path}/`),
    );
    if (shortcut) return shortcut.name + currentDir.slice(shortcut.browse_path.length);
    return currentDir ? `Workspace/${currentDir}` : "Workspace";
  }

  let destination = $derived(folderLabel(app.currentDir));
</script>

<div class="flex h-full flex-col bg-[var(--color-surface-muted)]">
  <header class="shrink-0 px-6 pb-4 pt-5">
    <h1 class="text-xl font-semibold tracking-tight">New from template</h1>
    <p class="mt-0.5 flex items-center gap-1 text-xs text-[var(--color-ink-muted)]">
      Start a document or project from a Typst Universe template. It is created in
      <Icon icon="ph:folder-simple" class="text-sm" />
      <span class="font-medium text-[var(--color-ink)]">{destination}</span>
    </p>
  </header>

  <div class="scroll-thin flex-1 overflow-y-auto px-6 pb-6">
    <TemplateBrowser onselect={(template) => (selected = template)} />
  </div>
</div>

{#if selected}
  <TemplateCreateModal template={selected} {oncreate} onclose={() => (selected = null)} />
{/if}
