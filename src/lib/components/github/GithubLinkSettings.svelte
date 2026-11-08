<script lang="ts">
  import { onMount, untrack } from "svelte";
  import Icon from "@iconify/svelte";
  import ConfirmModal from "../ConfirmModal.svelte";
  import {
    deleteGitLink,
    githubAccount,
    isGithubComUrl,
    parseExcludedPaths,
    updateGitLink,
    type GithubAccount,
    type GitLink,
    type GitLinkUpdate,
  } from "#lib/ts/github";

  let {
    path,
    link,
    onupdated,
    onunlinked,
  }: {
    path: string;
    link: GitLink;
    onupdated: (link: GitLink) => void;
    onunlinked: () => void;
  } = $props();

  let token = $state("");
  let excludedText = $state(untrack(() => link.excluded_paths.join("\n")));
  let busy = $state(false);
  let message = $state<{ kind: "success" | "error"; text: string } | null>(null);
  let confirmUnlink = $state(false);
  let account = $state<GithubAccount | null>(null);

  let canUseAccount = $derived(account !== null && !link.account_username && isGithubComUrl(link.repo_url));

  onMount(() => {
    githubAccount()
      .then((saved) => (account = saved))
      .catch(() => {});
  });

  async function save(update: GitLinkUpdate) {
    busy = true;
    message = null;
    try {
      onupdated(await updateGitLink(path, update));
      token = "";
      message = { kind: "success", text: "Saved." };
    } catch (err) {
      message = { kind: "error", text: String(err) };
    }
    busy = false;
  }

  function saveSettings() {
    const update: GitLinkUpdate = { excluded_paths: parseExcludedPaths(excludedText) };
    if (token.trim()) update.token = token;
    save(update);
  }

  async function unlink() {
    confirmUnlink = false;
    busy = true;
    message = null;
    try {
      await deleteGitLink(path);
      onunlinked();
    } catch (err) {
      message = { kind: "error", text: String(err) };
    }
    busy = false;
  }
</script>

<div class="flex flex-col gap-3 text-xs">
  <label class="flex flex-col gap-1.5">
    <span class="font-medium text-[var(--color-ink-muted)]">Access token</span>
    <input
      bind:value={token}
      type="password"
      autocomplete="off"
      placeholder={link.account_username
        ? "Enter a token to use instead of the connected account"
        : link.has_token
          ? "Saved. Enter a new token to replace it."
          : "No token saved"}
      class="field text-sm"
    />
    {#if link.account_username}
      <span class="flex items-center gap-1 text-[var(--color-ink-muted)]">
        <Icon icon="ph:user-check" class="text-sm" /> Uses the connected account {link.account_username}.
      </span>
    {:else if canUseAccount && account}
      <button type="button" onclick={() => save({ use_account: true })} disabled={busy} class="self-start font-semibold text-[var(--color-accent)] hover:underline disabled:opacity-60">
        Use my connected account {account.username} instead
      </button>
    {/if}
  </label>

  <label class="flex flex-col gap-1.5">
    <span class="font-medium text-[var(--color-ink-muted)]">Exclude paths</span>
    <textarea bind:value={excludedText} rows="3" placeholder={"data/\n*.csv"} class="field font-mono text-sm"></textarea>
  </label>

  {#if message}
    <p class={message.kind === "success" ? "text-[var(--color-success)]" : "text-[var(--color-danger)]"}>{message.text}</p>
  {/if}

  <div class="flex items-center justify-between gap-2">
    <div class="flex gap-2">
      <button class="btn btn-ghost text-[var(--color-danger)]" onclick={() => (confirmUnlink = true)} disabled={busy}>
        <Icon icon="ph:link-break" /> Unlink
      </button>
      {#if link.has_token}
        <button class="btn btn-ghost" onclick={() => save({ token: "" })} disabled={busy}>
          {link.account_username ? "Stop using account" : "Remove token"}
        </button>
      {/if}
    </div>
    <button class="btn btn-primary" onclick={saveSettings} disabled={busy}>
      <Icon icon={busy ? "ph:spinner" : "ph:floppy-disk"} class={busy ? "animate-spin" : ""} /> Save
    </button>
  </div>
</div>

{#if confirmUnlink}
  <ConfirmModal
    title="Unlink repository"
    message="Stop syncing this project with GitHub? The project files stay as they are, and nothing changes in the repository."
    confirmLabel="Unlink"
    onconfirm={unlink}
    onclose={() => (confirmUnlink = false)}
  />
{/if}
