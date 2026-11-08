<script lang="ts">
  import { onMount } from "svelte";
  import Icon from "@iconify/svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import GithubChangeList from "./GithubChangeList.svelte";
  import GithubLinkSettings from "./GithubLinkSettings.svelte";
  import { formatDate } from "#lib/ts/local-entries";
  import { gitChanges, pullGit, pushGit, shortCommit, type GitChanges, type GitLink, type GitResolution } from "#lib/ts/github";

  let {
    path,
    link,
    onlinkchanged,
    onchanged,
    onunlinked,
  }: {
    path: string;
    link: GitLink;
    onlinkchanged: (link: GitLink) => void;
    onchanged: () => void;
    onunlinked: () => void;
  } = $props();

  let changes = $state<GitChanges | null>(null);
  let selected = $state<string[]>([]);
  let commitMessage = $state("");
  let conflicts = $state<string[]>([]);
  let busy = $state<"load" | "pull" | "push" | null>(null);
  let notice = $state("");
  let error = $state("");
  let showSettings = $state(false);

  async function loadChanges() {
    busy = "load";
    error = "";
    try {
      changes = await gitChanges(path);
      onlinkchanged(changes.link);
      selected = changes.local.map((change) => change.path);
    } catch (err) {
      error = String(err);
    }
    busy = null;
  }

  async function pull(resolution: GitResolution = "ask") {
    busy = "pull";
    error = "";
    notice = "";
    try {
      const result = await pullGit(path, resolution);
      onlinkchanged(result.link);
      conflicts = result.applied ? [] : result.conflicts;
      if (result.applied) {
        const count = result.updated.length + result.removed.length;
        notice = count === 0 ? "Already up to date." : `Pulled ${count} ${count === 1 ? "file" : "files"}.`;
        if (result.too_large.length > 0) notice += ` Skipped ${result.too_large.join(", ")} (larger than 50 MB).`;
        onchanged();
      }
    } catch (err) {
      error = String(err);
    }
    busy = null;
    if (conflicts.length === 0) await loadChanges();
  }

  async function push() {
    busy = "push";
    error = "";
    notice = "";
    try {
      const result = await pushGit(path, commitMessage, selected);
      onlinkchanged(result.link);
      commitMessage = "";
      notice = `Pushed ${result.pushed.length} ${result.pushed.length === 1 ? "file" : "files"} as commit ${shortCommit(result.commit)}.`;
    } catch (err) {
      error = String(err);
    }
    busy = null;
    await loadChanges();
  }

  onMount(loadChanges);
</script>

<div class="flex flex-col gap-4 text-xs">
  <div class="flex items-start gap-3 rounded-lg border border-[var(--color-line)] p-3">
    <Icon icon="ph:github-logo" class="mt-0.5 shrink-0 text-lg text-[var(--color-ink-muted)]" />
    <div class="min-w-0 flex-1">
      <button class="block max-w-full truncate text-sm font-medium text-[var(--color-accent)] hover:underline" onclick={() => openUrl(link.repo_url)}>
        {link.repo_url}
      </button>
      <p class="truncate text-[var(--color-ink-muted)]">
        <span class="font-mono">{link.branch}</span>{#if link.folder}
          · <span class="font-mono">{link.folder}/</span>{/if}
        {#if link.base_commit}· <span class="font-mono">{shortCommit(link.base_commit)}</span>{/if}
        {#if link.last_synced_at}· Synced {formatDate(link.last_synced_at)}{/if}
        {#if link.account_username}· As {link.account_username}{/if}
      </p>
      {#if !link.has_token}
        <p class="mt-1 text-[var(--color-warning)]">No access token is saved, so changes cannot be pushed.</p>
      {/if}
    </div>
    <button class="icon-btn" onclick={loadChanges} disabled={busy !== null} title="Check for changes" aria-label="Check for changes">
      <Icon icon={busy === "load" ? "ph:spinner" : "ph:arrow-clockwise"} class={busy === "load" ? "animate-spin" : ""} />
    </button>
  </div>

  {#if conflicts.length > 0}
    <div class="flex flex-col gap-2 rounded-lg border border-[var(--color-warning)]/40 bg-[var(--color-warning)]/10 p-3">
      <p class="font-medium">These files changed both here and in the repository:</p>
      <p class="font-mono text-[var(--color-ink-muted)]">{conflicts.join(", ")}</p>
      <div class="flex flex-wrap justify-end gap-2">
        <button class="btn btn-secondary" onclick={() => pull("keep_mine")} disabled={busy !== null}>Keep my version</button>
        <button class="btn btn-primary" onclick={() => pull("keep_repository")} disabled={busy !== null}>Use repository version</button>
      </div>
    </div>
  {/if}

  {#if changes}
    <section class="flex flex-col gap-2">
      <div class="flex items-center justify-between">
        <h3 class="section-label">From the repository</h3>
        <button class="btn btn-secondary" onclick={() => pull()} disabled={busy !== null || !changes.behind}>
          <Icon icon={busy === "pull" ? "ph:spinner" : "ph:download-simple"} class={busy === "pull" ? "animate-spin" : ""} /> Pull
        </button>
      </div>
      {#if !changes.behind}
        <p class="text-[var(--color-ink-muted)]">Up to date with {link.branch}.</p>
      {:else if changes.remote.length === 0}
        <p class="text-[var(--color-ink-muted)]">New commits are available, but they do not change files in this project.</p>
      {:else}
        <GithubChangeList changes={changes.remote} conflicts={changes.conflicts} />
      {/if}
    </section>

    <section class="flex flex-col gap-2">
      <h3 class="section-label">Your changes</h3>
      {#if changes.local.length === 0}
        <p class="text-[var(--color-ink-muted)]">Nothing to commit.</p>
      {:else}
        <GithubChangeList changes={changes.local} conflicts={changes.conflicts} selectable bind:selected />
        <input bind:value={commitMessage} placeholder="Describe your changes" maxlength="5000" class="field text-sm" />
        {#if changes.behind}
          <p class="text-[var(--color-warning)]">Pull the new commits before pushing.</p>
        {/if}
        <div class="flex justify-end">
          <button
            class="btn btn-primary"
            onclick={push}
            disabled={busy !== null || changes.behind || selected.length === 0 || !commitMessage.trim() || !link.has_token}
          >
            <Icon icon={busy === "push" ? "ph:spinner" : "ph:upload-simple"} class={busy === "push" ? "animate-spin" : ""} />
            Commit and push {selected.length} {selected.length === 1 ? "file" : "files"}
          </button>
        </div>
      {/if}
    </section>
  {:else if busy === "load"}
    <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]">
      <Icon icon="ph:spinner" class="animate-spin" /> Checking the repository...
    </p>
  {/if}

  {#if notice}
    <p class="text-[var(--color-success)]">{notice}</p>
  {/if}
  {#if error}
    <p class="text-[var(--color-danger)]">{error}</p>
  {/if}

  <section class="flex flex-col gap-3 border-t border-[var(--color-line)] pt-3">
    <button class="flex items-center gap-1 self-start font-medium text-[var(--color-accent)] hover:underline" onclick={() => (showSettings = !showSettings)}>
      <Icon icon={showSettings ? "ph:caret-down" : "ph:caret-right"} /> Link settings
    </button>
    {#if showSettings}
      <GithubLinkSettings {path} {link} onupdated={onlinkchanged} {onunlinked} />
    {/if}
  </section>
</div>
