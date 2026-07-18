<script lang="ts">
  import Icon from "@iconify/svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import Modal from "./Modal.svelte";
  import * as api from "$lib/ts/api";
  import { app, applyTheme, refreshEntries, setError } from "$lib/ts/state.svelte";

  interface Props {
    onclose: () => void;
    onsignin: () => void;
  }

  let { onclose, onsignin }: Props = $props();

  let workspaceRoot = $state(app.settings?.workspace_root ?? "");
  let serverUrl = $state(app.settings?.server_url ?? "");
  let saving = $state(false);

  async function browse() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      workspaceRoot = selected;
    }
  }

  async function save() {
    saving = true;
    try {
      app.settings = await api.updateSettings({
        workspaceRoot,
        serverUrl,
      });
      await refreshEntries();
      onclose();
    } catch (error) {
      setError(error);
    } finally {
      saving = false;
    }
  }

  async function signOut() {
    try {
      await api.cloudLogout();
      app.account = null;
      app.spaces = [];
      app.settings = await api.getSettings();
    } catch (error) {
      setError(error);
    }
  }
</script>

<Modal title="Settings" icon="ph:gear-six" {onclose}>
  <div class="flex flex-col gap-5">
    <div class="flex flex-col gap-1 text-xs">
      <span class="font-medium text-[var(--color-ink-muted)]">Workspace folder</span>
      <div class="flex gap-2">
        <input
          class="flex-1 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none"
          bind:value={workspaceRoot}
        />
        <button
          class="flex items-center gap-1 rounded-md border border-[var(--color-line)] px-3 text-xs hover:bg-[var(--color-surface-muted)]"
          onclick={browse}
        >
          <Icon icon="ph:folder-open" />
          Browse
        </button>
      </div>
      <span class="text-[var(--color-ink-muted)]">
        Projects are stored as plain folders here.
      </span>
    </div>

    <div class="flex flex-col gap-1 text-xs">
      <span class="font-medium text-[var(--color-ink-muted)]">TypstDrive server</span>
      <input
        class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none"
        bind:value={serverUrl}
      />
    </div>

    <div
      class="flex items-center gap-3 rounded-md border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-3"
    >
      <Icon
        icon={app.account ? "ph:user-circle-check" : "ph:user-circle"}
        class="text-2xl {app.account ? 'text-[var(--color-success)]' : 'text-[var(--color-ink-muted)]'}"
      />
      <div class="flex-1 text-xs">
        {#if app.account}
          <p class="font-medium">{app.account.username}</p>
          <p class="text-[var(--color-ink-muted)]">{app.account.email}</p>
        {:else}
          <p class="font-medium">Not connected</p>
          <p class="text-[var(--color-ink-muted)]">
            Sign in to sync projects to the cloud.
          </p>
        {/if}
      </div>
      {#if app.account}
        <button
          class="rounded-md border border-[var(--color-line)] px-3 py-1.5 text-xs hover:bg-[var(--color-surface)]"
          onclick={signOut}
        >
          Sign out
        </button>
      {:else}
        <button
          class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white hover:opacity-90"
          onclick={onsignin}
        >
          Sign in
        </button>
      {/if}
    </div>

    <div class="flex items-center justify-between text-xs">
      <span class="font-medium text-[var(--color-ink-muted)]">Appearance</span>
      <div class="flex gap-1">
        {#each [["light", "ph:sun"], ["dark", "ph:moon"]] as [value, icon]}
          <button
            class="flex items-center gap-1.5 rounded-md border px-3 py-1.5 transition
              {app.theme === value
              ? 'border-[var(--color-accent)] bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
              : 'border-[var(--color-line)] text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)]'}"
            onclick={() => applyTheme(value as "light" | "dark")}
          >
            <Icon {icon} />
            {value === "light" ? "Light" : "Dark"}
          </button>
        {/each}
      </div>
    </div>
  </div>

  {#snippet footer()}
    <button
      class="rounded-md px-3 py-1.5 text-xs text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]"
      onclick={onclose}
    >
      Cancel
    </button>
    <button
      class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90 disabled:opacity-50"
      disabled={saving}
      onclick={save}
    >
      Save
    </button>
  {/snippet}
</Modal>
