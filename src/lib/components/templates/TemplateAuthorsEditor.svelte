<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { TemplateAuthor } from "#lib/ts/templates";

  let { authors = $bindable() }: { authors: TemplateAuthor[] } = $props();

  function addAuthor() {
    authors = [...authors, { name: "", affiliation: "" }];
  }

  function removeAuthor(index: number) {
    authors = authors.filter((_, position) => position !== index);
  }
</script>

<div class="flex flex-col gap-2 text-xs">
  <div>
    <p class="font-medium text-[var(--color-ink-muted)]">Authors <span class="font-normal">(optional)</span></p>
    <p class="text-[var(--color-ink-muted)]">Fills in the template's author names and affiliations.</p>
  </div>

  {#each authors as author, index (index)}
    <div class="flex items-center gap-1.5">
      <input bind:value={author.name} placeholder="Full name" aria-label="Author name" class="field field-sm min-w-0 flex-1" />
      <input bind:value={author.affiliation} placeholder="Affiliation" aria-label="Affiliation" class="field field-sm min-w-0 flex-1" />
      <button type="button" class="icon-btn icon-btn-sm" onclick={() => removeAuthor(index)} title="Remove author" aria-label="Remove author">
        <Icon icon="ph:x" />
      </button>
    </div>
  {/each}

  <button type="button" class="btn btn-ghost self-start" onclick={addAuthor}>
    <Icon icon="ph:user-plus" />
    Add author
  </button>
</div>
