<script lang="ts">
  import { onMount } from "svelte";
  import Icon from "@iconify/svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import SettingsHeader from "./SettingsHeader.svelte";
  import SettingsCard from "./SettingsCard.svelte";
  import SettingsRow from "./SettingsRow.svelte";
  import { connectZotero, disconnectZotero, zoteroStatus, type ZoteroStatus } from "$lib/ts/citations";

  interface Props {
    fieldClass: string;
  }

  let { fieldClass }: Props = $props();

  const KEY_URL = "https://www.zotero.org/settings/keys/new";

  let status = $state<ZoteroStatus | null>(null);
  let apiKey = $state("");
  let busy = $state(false);
  let error = $state("");

  onMount(async () => {
    try {
      status = await zoteroStatus();
    } catch (err) {
      error = String(err);
    }
  });

  async function connect() {
    if (!apiKey.trim()) return;
    busy = true;
    error = "";
    try {
      status = await connectZotero(apiKey.trim());
      apiKey = "";
    } catch (err) {
      error = String(err);
    }
    busy = false;
  }

  async function disconnect() {
    busy = true;
    error = "";
    try {
      await disconnectZotero();
      status = { connected: false, username: null };
    } catch (err) {
      error = String(err);
    }
    busy = false;
  }
</script>

<SettingsHeader
  title="Citations"
  description="Connect Zotero to sync libraries and collections into bibliography files. Add references by DOI or arXiv ID from the editor toolbar."
  icon="ph:quotes"
/>
<div class="flex flex-col gap-4">
  <SettingsCard title="Zotero">
    {#if !status}
      <p class="px-4 py-3 text-xs text-[var(--color-ink-muted)]">Loading...</p>
    {:else if status.connected}
      <SettingsRow label="Connected account" description="Syncs keep using this account. The key is stored on this computer.">
        <span class="text-[13px] font-medium">{status.username}</span>
        <button class="btn btn-secondary" disabled={busy} onclick={disconnect}>Disconnect</button>
      </SettingsRow>
    {:else}
      <SettingsRow label="API key" description="A key with read access to your library and any groups you want to sync." stacked>
        <div class="flex w-full gap-2">
          <input type="password" bind:value={apiKey} autocomplete="off" placeholder="Paste your Zotero API key" class="{fieldClass} min-w-0 flex-1" />
          <button class="btn btn-primary" disabled={busy || !apiKey.trim()} onclick={connect}>
            {busy ? "Connecting..." : "Connect"}
          </button>
        </div>
      </SettingsRow>
      <button class="flex items-center gap-1 px-4 py-3 text-xs font-medium text-[var(--color-accent)] hover:underline" onclick={() => openUrl(KEY_URL)}>
        Create a key on zotero.org
        <Icon icon="ph:arrow-square-out" />
      </button>
    {/if}
  </SettingsCard>
  {#if error}
    <p class="text-xs text-[var(--color-danger)]">{error}</p>
  {/if}
</div>
