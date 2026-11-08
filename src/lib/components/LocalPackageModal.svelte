<script lang="ts">
  import Icon from "@iconify/svelte";
  import { onMount } from "svelte";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import Modal from "./Modal.svelte";
  import {
    installLocalPackage,
    localPackageInfo,
    removeLocalPackage,
    type LocalPackageInfo,
  } from "#lib/ts/local-packages";

  let { path, onclose }: { path: string; onclose: () => void } = $props();

  let info = $state<LocalPackageInfo | null>(null);
  let problem = $state("");
  let success = $state("");
  let installing = $state(false);
  let copied = $state(false);

  let isInstalled = $derived(info?.installed_versions.includes(info.version) ?? false);

  onMount(async () => {
    try {
      info = await localPackageInfo(path);
    } catch (error) {
      problem = String(error);
    }
  });

  async function install() {
    if (!info || installing) return;
    installing = true;
    problem = "";
    success = "";
    try {
      info = await installLocalPackage(path);
      success = `Installed @local/${info.name}:${info.version}`;
    } catch (error) {
      problem = String(error);
    }
    installing = false;
  }

  async function remove(version: string) {
    if (!info) return;
    problem = "";
    success = "";
    try {
      info = { ...info, installed_versions: await removeLocalPackage(info.name, version) };
    } catch (error) {
      problem = String(error);
    }
  }

  async function copyImport() {
    if (!info) return;
    await writeText(info.import_line);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }
</script>

<Modal title="Install as @local package" icon="ph:package" {onclose}>
  <div class="flex flex-col gap-4 text-xs">
    <p class="text-[var(--color-ink-muted)]">
      Copies this project into your Typst package folder so any document can import it from the
      <code class="rounded bg-[var(--color-surface-sunken)] px-1 py-0.5 font-mono">@local</code> namespace.
      The name, version and entrypoint come from the project's
      <code class="rounded bg-[var(--color-surface-sunken)] px-1 py-0.5 font-mono">typst.toml</code>.
    </p>

    {#if info}
      <div class="flex flex-col gap-1.5">
        <span class="font-medium text-[var(--color-ink-muted)]">Import with</span>
        <div class="field flex items-center gap-2 p-0">
          <code class="min-w-0 flex-1 truncate px-3 py-2 font-mono">{info.import_line}</code>
          <button class="icon-btn icon-btn-sm mr-1.5" onclick={copyImport} title="Copy import" aria-label="Copy import">
            <Icon icon={copied ? "ph:check" : "ph:copy"} />
          </button>
        </div>
        {#if isInstalled}
          <p class="text-[var(--color-ink-muted)]">Version {info.version} is already installed. Installing again replaces it.</p>
        {/if}
      </div>

      {#if info.installed_versions.length > 0}
        <div class="flex flex-col gap-1.5">
          <span class="font-medium text-[var(--color-ink-muted)]">Installed versions</span>
          <ul class="flex flex-col divide-y divide-[var(--color-line)] rounded-md border border-[var(--color-line)]">
            {#each info.installed_versions as version (version)}
              <li class="flex items-center gap-2 px-3 py-1.5">
                <span class="flex-1 font-mono">@local/{info.name}:{version}</span>
                <button class="icon-btn icon-btn-sm" onclick={() => remove(version)} title="Remove version" aria-label="Remove version {version}">
                  <Icon icon="ph:trash" />
                </button>
              </li>
            {/each}
          </ul>
        </div>
      {/if}
    {/if}

    {#if problem}
      <p class="break-words text-[var(--color-danger)]">{problem}</p>
    {/if}
    {#if success}
      <p class="text-[var(--color-success)]">{success}</p>
    {/if}
  </div>

  {#snippet footer()}
    <button class="btn btn-ghost" onclick={onclose}>Close</button>
    <button class="btn btn-primary" onclick={install} disabled={!info || installing}>
      {#if installing}<Icon icon="ph:spinner" class="animate-spin" />{/if}
      {isInstalled ? "Reinstall" : "Install"}
    </button>
  {/snippet}
</Modal>
