<script lang="ts">
  import { onMount, type Snippet } from "svelte";
  import Icon from "@iconify/svelte";
  import GithubImportPreview from "./GithubImportPreview.svelte";
  import {
    githubAccount,
    isGithubComUrl,
    parseExcludedPaths,
    type GithubAccount,
    type GitImportPreview,
    type GitLinkRequest,
  } from "#lib/ts/github";

  let {
    onpreview,
    onsubmit,
    submitLabel,
    intro,
    repoUrl = $bindable(""),
    showReplaceOption = true,
    ready = true,
    children,
  }: {
    onpreview: (request: GitLinkRequest) => Promise<GitImportPreview>;
    onsubmit: (request: GitLinkRequest) => Promise<void>;
    submitLabel: string;
    intro: string;
    repoUrl?: string;
    showReplaceOption?: boolean;
    ready?: boolean;
    children?: Snippet;
  } = $props();

  let branch = $state("main");
  let folder = $state("");
  let token = $state("");
  let excludedText = $state("");
  let replaceProjectFiles = $state(true);
  let showAdvanced = $state(false);
  let preview = $state<GitImportPreview | null>(null);
  let account = $state<GithubAccount | null>(null);
  let busy = $state<"preview" | "link" | null>(null);
  let error = $state("");

  let usesAccount = $derived(!token.trim() && account !== null && isGithubComUrl(repoUrl));
  let canSubmit = $derived(ready && repoUrl.trim().length > 0 && branch.trim().length > 0 && busy === null);

  function linkRequest(): GitLinkRequest {
    return {
      repo_url: repoUrl,
      branch,
      folder,
      token,
      excluded_paths: parseExcludedPaths(excludedText),
      replace_project_files: showReplaceOption && replaceProjectFiles,
    };
  }

  $effect(() => {
    linkRequest();
    preview = null;
  });

  onMount(() => {
    githubAccount()
      .then((saved) => (account = saved))
      .catch(() => {});
  });

  async function checkRepository() {
    busy = "preview";
    error = "";
    try {
      preview = await onpreview(linkRequest());
    } catch (err) {
      error = String(err);
    }
    busy = null;
  }

  async function submit() {
    busy = "link";
    error = "";
    try {
      await onsubmit(linkRequest());
    } catch (err) {
      error = String(err);
    }
    busy = null;
  }
</script>

<div class="flex flex-col gap-4 text-xs">
  <p class="text-[var(--color-ink-muted)]">{intro}</p>

  {@render children?.()}

  <label class="flex flex-col gap-1.5">
    <span class="font-medium text-[var(--color-ink-muted)]">Repository address</span>
    <input bind:value={repoUrl} placeholder="https://github.com/owner/repository" class="field text-sm" />
  </label>

  <div class="grid gap-3 sm:grid-cols-2">
    <label class="flex flex-col gap-1.5">
      <span class="font-medium text-[var(--color-ink-muted)]">Branch</span>
      <input bind:value={branch} placeholder="main" class="field font-mono text-sm" />
    </label>
    <label class="flex flex-col gap-1.5">
      <span class="font-medium text-[var(--color-ink-muted)]">Folder in the repository</span>
      <input bind:value={folder} placeholder="Whole repository" class="field font-mono text-sm" />
    </label>
  </div>

  <label class="flex flex-col gap-1.5">
    <span class="font-medium text-[var(--color-ink-muted)]">Access token</span>
    <input
      bind:value={token}
      type="password"
      autocomplete="off"
      placeholder={account ? "Leave empty to use your connected account" : "Needed for private repositories and for pushing"}
      class="field text-sm"
    />
    {#if usesAccount && account}
      <span class="flex items-center gap-1 text-[var(--color-success)]">
        <Icon icon="ph:user-check" class="text-sm" /> Uses your connected account {account.username}.
      </span>
    {:else}
      <span class="text-[var(--color-ink-muted)]">
        Use a token with read and write access to repository contents, or connect an account in Settings, GitHub. It is stored on this computer.
      </span>
    {/if}
  </label>

  <button type="button" onclick={() => (showAdvanced = !showAdvanced)} class="flex items-center gap-1 self-start font-medium text-[var(--color-accent)] hover:underline">
    <Icon icon={showAdvanced ? "ph:caret-down" : "ph:caret-right"} /> More options
  </button>

  {#if showAdvanced}
    <div class="flex flex-col gap-3">
      <label class="flex flex-col gap-1.5">
        <span class="font-medium text-[var(--color-ink-muted)]">Exclude paths</span>
        <textarea bind:value={excludedText} rows="3" placeholder={"data/\n*.csv"} class="field font-mono text-sm"></textarea>
        <span class="text-[var(--color-ink-muted)]">One per line. A folder like data/ or a pattern like *.csv.</span>
      </label>
      {#if showReplaceOption}
        <label class="flex items-center gap-2">
          <input type="checkbox" bind:checked={replaceProjectFiles} />
          Remove project files that are not in the repository
        </label>
      {/if}
    </div>
  {/if}

  {#if preview}
    <GithubImportPreview {preview} />
  {/if}

  {#if error}
    <p class="text-[var(--color-danger)]">{error}</p>
  {/if}

  <div class="flex justify-end gap-2">
    <button class="btn btn-secondary" onclick={checkRepository} disabled={!canSubmit}>
      <Icon icon={busy === "preview" ? "ph:spinner" : "ph:magnifying-glass"} class={busy === "preview" ? "animate-spin" : ""} />
      Check repository
    </button>
    <button class="btn btn-primary" onclick={submit} disabled={!canSubmit || !preview}>
      <Icon icon={busy === "link" ? "ph:spinner" : "ph:git-branch"} class={busy === "link" ? "animate-spin" : ""} />
      {submitLabel}
    </button>
  </div>
</div>
