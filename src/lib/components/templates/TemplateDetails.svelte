<script lang="ts">
  import Icon from "@iconify/svelte";
  import { untrack } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import TemplateThumbnail from "./TemplateThumbnail.svelte";
  import TemplateAuthorsEditor from "./TemplateAuthorsEditor.svelte";
  import { focusOnMount } from "#lib/ts/focus-on-mount";
  import {
    categoryLabel,
    loadSavedAuthors,
    readableName,
    saveAuthors,
    universeUrl,
    type TemplateAuthor,
    type UniverseTemplate,
  } from "#lib/ts/templates";

  let {
    template,
    title = $bindable(),
    authors = $bindable(),
    oncreate,
  }: {
    template: UniverseTemplate;
    title: string;
    authors: TemplateAuthor[];
    oncreate: () => void;
  } = $props();

  untrack(() => {
    if (!title) title = readableName(template.name);
    if (authors.length === 0) authors = loadSavedAuthors();
  });

  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!title.trim()) return;
    saveAuthors(authors.filter((author) => author.name.trim()));
    oncreate();
  }
</script>

<div class="grid gap-5 sm:grid-cols-[12rem_1fr]">
  <TemplateThumbnail {template} class="aspect-[3/4] w-full rounded-md border border-[var(--color-line)]" />

  <div class="flex min-w-0 flex-col gap-3">
    <p class="text-sm">{template.description}</p>

    <dl class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5 text-xs">
      <dt class="text-[var(--color-ink-muted)]">Package</dt>
      <dd class="font-mono">@preview/{template.name}:{template.version}</dd>
      {#if template.authors.length > 0}
        <dt class="text-[var(--color-ink-muted)]">By</dt>
        <dd>{template.authors.join(", ")}</dd>
      {/if}
      {#if template.categories.length > 0}
        <dt class="text-[var(--color-ink-muted)]">Type</dt>
        <dd>{template.categories.map(categoryLabel).join(", ")}</dd>
      {/if}
    </dl>

    <button
      type="button"
      onclick={() => openUrl(universeUrl(template))}
      class="flex items-center gap-1 self-start text-xs text-[var(--color-accent)] hover:underline"
    >
      View on Typst Universe <Icon icon="ph:arrow-square-out" class="text-sm" />
    </button>

    <form id="template-create-form" onsubmit={submit} class="mt-auto flex flex-col gap-1.5 pt-2 text-xs">
      <label for="template-title" class="font-medium text-[var(--color-ink-muted)]">Name</label>
      <input id="template-title" use:focusOnMount bind:value={title} maxlength="200" class="field text-sm" />
      <div class="mt-2">
        <TemplateAuthorsEditor bind:authors />
      </div>
    </form>
  </div>
</div>
