<script lang="ts">
  import { onMount } from "svelte";
  import Icon from "@iconify/svelte";
  import Modal from "../Modal.svelte";
  import GithubLinkForm from "./GithubLinkForm.svelte";
  import GithubSyncPanel from "./GithubSyncPanel.svelte";
  import { createGitLink, gitLinkStatus, previewGitLink, type GitLink, type GitLinkRequest } from "#lib/ts/github";

  let {
    path,
    onchanged,
    onclose,
  }: {
    path: string;
    onchanged: () => void;
    onclose: () => void;
  } = $props();

  let link = $state<GitLink | null>(null);
  let loaded = $state(false);
  let error = $state("");

  async function load() {
    error = "";
    try {
      link = await gitLinkStatus(path);
    } catch (err) {
      error = String(err);
    }
    loaded = true;
  }

  async function createLink(request: GitLinkRequest) {
    link = await createGitLink(path, request);
    onchanged();
  }

  onMount(load);
</script>

<Modal title="GitHub" icon="ph:github-logo" width="max-w-[620px]" {onclose}>
  {#if error}
    <p class="text-sm text-[var(--color-danger)]">{error}</p>
  {:else if !loaded}
    <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="ph:spinner" class="animate-spin" /> Loading...</p>
  {:else if link}
    <GithubSyncPanel {path} {link} onlinkchanged={(updated) => (link = updated)} {onchanged} onunlinked={load} />
  {:else}
    <GithubLinkForm
      intro="Link this project to a folder in a GitHub repository. Only document sources, bibliographies, images and fonts are copied into the project. Everything else stays in the repository untouched, so large repositories are fine."
      submitLabel="Link and import"
      onpreview={(request) => previewGitLink(path, request)}
      onsubmit={createLink}
    />
  {/if}

  {#snippet footer()}
    <button class="btn btn-primary" onclick={onclose}>Done</button>
  {/snippet}
</Modal>
