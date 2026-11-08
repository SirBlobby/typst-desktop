<script lang="ts">
  import { onMount } from "svelte";
  import Icon from "@iconify/svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import SettingsHeader from "./SettingsHeader.svelte";
  import SettingsCard from "./SettingsCard.svelte";
  import SettingsRow from "./SettingsRow.svelte";
  import { connectGithub, disconnectGithub, githubAccount, TOKEN_PAGE_URL, type GithubAccount } from "#lib/ts/github";

  let { fieldClass }: { fieldClass: string } = $props();

  let account = $state<GithubAccount | null>(null);
  let loaded = $state(false);
  let token = $state("");
  let busy = $state(false);
  let error = $state("");

  onMount(async () => {
    try {
      account = await githubAccount();
    } catch (err) {
      error = String(err);
    }
    loaded = true;
  });

  async function connect() {
    if (!token.trim()) return;
    busy = true;
    error = "";
    try {
      account = await connectGithub(token.trim());
      token = "";
    } catch (err) {
      error = String(err);
    }
    busy = false;
  }

  async function disconnect() {
    busy = true;
    error = "";
    try {
      await disconnectGithub();
      account = null;
    } catch (err) {
      error = String(err);
    }
    busy = false;
  }
</script>

<SettingsHeader
  title="GitHub"
  description="Link projects to a folder in a GitHub repository, then pull and push changes. A saved account is used for every link that has no token of its own."
  icon="ph:github-logo"
/>
<div class="flex flex-col gap-4">
  <SettingsCard title="Account">
    {#if !loaded}
      <p class="px-4 py-3 text-xs text-[var(--color-ink-muted)]">Loading...</p>
    {:else if account}
      <SettingsRow label="Connected account" description="Links without their own token use this account. The token is stored on this computer.">
        <span class="text-[13px] font-medium">{account.username}</span>
        <button class="btn btn-secondary" disabled={busy} onclick={disconnect}>Disconnect</button>
      </SettingsRow>
    {:else}
      <SettingsRow label="Access token" description="A fine-grained token with read and write access to the contents of the repositories you want to sync." stacked>
        <div class="flex w-full gap-2">
          <input type="password" bind:value={token} autocomplete="off" placeholder="Paste your GitHub token" class="{fieldClass} min-w-0 flex-1" />
          <button class="btn btn-primary" disabled={busy || !token.trim()} onclick={connect}>
            {busy ? "Connecting..." : "Connect"}
          </button>
        </div>
      </SettingsRow>
      <button class="flex items-center gap-1 px-4 py-3 text-xs font-medium text-[var(--color-accent)] hover:underline" onclick={() => openUrl(TOKEN_PAGE_URL)}>
        Create a token on github.com
        <Icon icon="ph:arrow-square-out" />
      </button>
    {/if}
  </SettingsCard>
  {#if error}
    <p class="text-xs text-[var(--color-danger)]">{error}</p>
  {/if}
</div>
