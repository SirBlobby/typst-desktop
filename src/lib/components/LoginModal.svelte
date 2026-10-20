<script lang="ts">
  import { untrack } from "svelte";
  import Icon from "@iconify/svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Modal from "./Modal.svelte";
  import * as api from "$lib/ts/api";

  interface Props {
    serverUrl: string;
    onsuccess: () => void;
    onclose: () => void;
  }

  let { serverUrl, onsuccess, onclose }: Props = $props();

  let url = $state(untrack(() => serverUrl));
  let code = $state("");
  let busy = $state(false);
  let error = $state("");
  let signInPageOpened = $state(false);

  const inputClass =
    "field";

  async function openSignInPage() {
    if (!url.trim()) {
      error = "Enter your TypstDrive server address";
      return;
    }
    error = "";
    try {
      await openUrl(await api.cloudSignInUrl(url.trim()));
      signInPageOpened = true;
    } catch (caught) {
      error = api.errorMessage(caught);
    }
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!url.trim()) {
      error = "Enter your TypstDrive server address";
      return;
    }
    if (!code.trim()) {
      error = "Enter the sign-in key shown in your browser";
      return;
    }

    busy = true;
    error = "";
    try {
      await api.cloudLogin(url.trim(), code.trim());
      onsuccess();
    } catch (caught) {
      error = api.errorMessage(caught);
    } finally {
      busy = false;
    }
  }
</script>

<Modal title="Connect to TypstDrive" icon="ph:cloud" {onclose}>
  <form id="login-form" class="flex flex-col gap-4" onsubmit={submit}>
    <label class="flex flex-col gap-1 text-xs">
      <span class="font-medium text-[var(--color-ink-muted)]">Server</span>
      <input class={inputClass} bind:value={url} placeholder="https://drive.example.com" />
    </label>

    <div class="flex flex-col gap-2 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-3">
      <p class="text-xs text-[var(--color-ink-muted)]">
        Sign in to TypstDrive in your browser. It will show a one-time key you can paste below.
      </p>
      <button
        type="button"
        class="btn btn-secondary h-8 w-full"
        onclick={openSignInPage}
      >
        <Icon icon="ph:arrow-square-out" class="text-sm" />
        {signInPageOpened ? "Open sign-in page again" : "Open sign-in page"}
      </button>
    </div>

    <label class="flex flex-col gap-1 text-xs">
      <span class="font-medium text-[var(--color-ink-muted)]">One-time key</span>
      <input
        class="{inputClass} font-mono uppercase tracking-widest"
        bind:value={code}
        placeholder="XXXX-XXXX"
        autocomplete="off"
        spellcheck="false"
      />
    </label>

    {#if error}
      <p class="text-xs text-[var(--color-danger)]">{error}</p>
    {/if}
  </form>

  {#snippet footer()}
    <button
      class="btn btn-ghost"
      onclick={onclose}
    >
      Cancel
    </button>
    <button
      type="submit"
      form="login-form"
      class="btn btn-primary"
      disabled={busy}
    >
      {busy ? "Connecting..." : "Connect"}
    </button>
  {/snippet}
</Modal>
