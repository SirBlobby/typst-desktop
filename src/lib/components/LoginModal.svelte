<script lang="ts">
  import Modal from "./Modal.svelte";
  import * as api from "$lib/ts/api";

  interface Props {
    serverUrl: string;
    onsuccess: () => void;
    onclose: () => void;
  }

  let { serverUrl, onsuccess, onclose }: Props = $props();

  let url = $state(serverUrl);
  let email = $state("");
  let password = $state("");
  let busy = $state(false);
  let error = $state("");

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!email.trim() || !password) {
      error = "Enter your email and password";
      return;
    }

    busy = true;
    error = "";
    try {
      await api.cloudLogin(url.trim(), email.trim(), password);
      onsuccess();
    } catch (caught) {
      error = api.errorMessage(caught);
    } finally {
      busy = false;
    }
  }
</script>

<Modal title="Connect to TypstDrive" icon="ph:cloud" {onclose}>
  <form id="login-form" class="flex flex-col gap-3" onsubmit={submit}>
    <label class="flex flex-col gap-1 text-xs">
      <span class="font-medium text-[var(--color-ink-muted)]">Server</span>
      <input
        class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none"
        bind:value={url}
        placeholder="https://drive.example.com"
      />
    </label>

    <label class="flex flex-col gap-1 text-xs">
      <span class="font-medium text-[var(--color-ink-muted)]">Email</span>
      <input
        type="email"
        class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none"
        bind:value={email}
      />
    </label>

    <label class="flex flex-col gap-1 text-xs">
      <span class="font-medium text-[var(--color-ink-muted)]">Password</span>
      <input
        type="password"
        class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none"
        bind:value={password}
      />
    </label>

    {#if error}
      <p class="text-xs text-[var(--color-danger)]">{error}</p>
    {/if}
  </form>

  {#snippet footer()}
    <button
      class="rounded-md px-3 py-1.5 text-xs text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]"
      onclick={onclose}
    >
      Cancel
    </button>
    <button
      type="submit"
      form="login-form"
      class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90 disabled:opacity-50"
      disabled={busy}
    >
      {busy ? "Connecting..." : "Connect"}
    </button>
  {/snippet}
</Modal>
