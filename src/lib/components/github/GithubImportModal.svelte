<script lang="ts">
  import Modal from "../Modal.svelte";
  import GithubLinkForm from "./GithubLinkForm.svelte";
  import { importGitProject, previewGitImport, repositoryName, type GitLinkRequest } from "#lib/ts/github";

  let {
    parent,
    onimported,
    onclose,
  }: {
    parent: string;
    onimported: (path: string) => void;
    onclose: () => void;
  } = $props();

  let repoUrl = $state("");
  let projectName = $state("");
  let nameEdited = $state(false);

  $effect(() => {
    const suggested = repositoryName(repoUrl);
    if (!nameEdited) projectName = suggested;
  });

  async function importProject(request: GitLinkRequest) {
    onimported(await importGitProject(parent, projectName.trim(), request));
  }
</script>

<Modal title="Import from GitHub" icon="ph:github-logo" width="max-w-[620px]" {onclose}>
  <GithubLinkForm
    intro="Create a project from a folder in a GitHub repository and keep it in sync. Only document sources, bibliographies, images and fonts are copied. Everything else stays in the repository untouched, so large repositories are fine."
    submitLabel="Import project"
    showReplaceOption={false}
    ready={projectName.trim().length > 0}
    onpreview={previewGitImport}
    onsubmit={importProject}
    bind:repoUrl
  >
    <label class="flex flex-col gap-1.5">
      <span class="font-medium text-[var(--color-ink-muted)]">Project name</span>
      <input
        bind:value={projectName}
        oninput={() => (nameEdited = projectName.trim().length > 0)}
        placeholder="My paper"
        class="field text-sm"
      />
    </label>
  </GithubLinkForm>

  {#snippet footer()}
    <button class="btn btn-secondary" onclick={onclose}>Cancel</button>
  {/snippet}
</Modal>
