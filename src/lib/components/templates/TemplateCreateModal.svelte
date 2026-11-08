<script lang="ts">
  import Icon from "@iconify/svelte";
  import { untrack } from "svelte";
  import Modal from "../Modal.svelte";
  import TemplateDetails from "./TemplateDetails.svelte";
  import { readableName, type TemplateAuthor, type UniverseTemplate } from "#lib/ts/templates";

  let {
    template,
    oncreate,
    onclose,
  }: {
    template: UniverseTemplate;
    oncreate: (template: UniverseTemplate, title: string, authors: TemplateAuthor[]) => Promise<void>;
    onclose: () => void;
  } = $props();

  let title = $state(untrack(() => readableName(template.name)));
  let authors = $state<TemplateAuthor[]>([]);
  let creating = $state(false);
  let problem = $state("");

  async function create() {
    if (!title.trim() || creating) return;
    creating = true;
    problem = "";
    try {
      await oncreate(template, title.trim(), authors.filter((author) => author.name.trim()));
    } catch (error) {
      problem = String(error);
      creating = false;
    }
  }
</script>

<Modal title={readableName(template.name)} icon="ph:squares-four" width="max-w-2xl" {onclose}>
  <TemplateDetails {template} bind:title bind:authors oncreate={create} />

  {#snippet footer()}
    {#if problem}
      <p class="mr-auto text-xs text-[var(--color-danger)]">{problem}</p>
    {/if}
    <button class="btn btn-ghost" onclick={onclose} disabled={creating}>Cancel</button>
    <button type="submit" form="template-create-form" class="btn btn-primary" disabled={!title.trim() || creating}>
      {#if creating}<Icon icon="ph:spinner" class="animate-spin" />{/if}
      {creating ? "Creating..." : "Create"}
    </button>
  {/snippet}
</Modal>
