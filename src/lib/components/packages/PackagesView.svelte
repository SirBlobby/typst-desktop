<script lang="ts">
  import Icon from "@iconify/svelte";
  import { onMount } from "svelte";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import PackageCard from "./PackageCard.svelte";
  import ConfirmModal from "../ConfirmModal.svelte";
  import CloudEmptyState from "../cloud/CloudEmptyState.svelte";
  import { setError, setStatus } from "#lib/ts/state.svelte";
  import {
    listLocalPackages,
    localPackagesFolder,
    matchesPackageQuery,
    removeLocalPackage,
    type LocalPackage,
  } from "#lib/ts/local-packages";

  let packages = $state<LocalPackage[]>([]);
  let loading = $state(true);
  let searchQuery = $state("");
  let pendingRemoval = $state<{ name: string; version: string } | null>(null);

  let visiblePackages = $derived(packages.filter((pack) => matchesPackageQuery(pack, searchQuery)));

  async function load() {
    loading = true;
    try {
      packages = await listLocalPackages();
    } catch (error) {
      setError(error);
    }
    loading = false;
  }

  async function openFolder() {
    try {
      await revealItemInDir(await localPackagesFolder());
    } catch (error) {
      setError(error);
    }
  }

  async function confirmRemoval() {
    if (!pendingRemoval) return;
    const { name, version } = pendingRemoval;
    pendingRemoval = null;
    try {
      await removeLocalPackage(name, version);
      setStatus(`Removed @local/${name}:${version}`);
      await load();
    } catch (error) {
      setError(error);
    }
  }

  onMount(load);
</script>

<div class="flex h-full flex-col bg-[var(--color-surface-muted)]">
  <header class="shrink-0 px-6 pb-4 pt-5">
    <div class="flex flex-wrap items-center gap-2">
      <div class="min-w-0 flex-1">
        <h1 class="truncate text-xl font-semibold tracking-tight">Packages</h1>
        <p class="text-xs text-[var(--color-ink-muted)]">Packages installed in the @local namespace on this computer.</p>
      </div>

      <div class="relative">
        <Icon icon="ph:magnifying-glass" class="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-sm text-[var(--color-ink-subtle)]" />
        <input
          type="search"
          bind:value={searchQuery}
          placeholder="Filter packages"
          aria-label="Filter packages"
          class="field field-sm h-7 w-56 pl-8"
        />
      </div>

      <button class="btn btn-secondary" onclick={openFolder}>
        <Icon icon="ph:folder-open" />
        Open folder
      </button>
      <button class="btn btn-secondary" onclick={load} aria-label="Refresh packages">
        <Icon icon="ph:arrow-clockwise" class={loading ? "animate-spin" : ""} />
        Refresh
      </button>
    </div>
  </header>

  <div class="scroll-thin flex-1 overflow-y-auto px-6 pb-6">
    {#if loading && packages.length === 0}
      <p class="flex items-center gap-2 py-10 text-sm text-[var(--color-ink-muted)]">
        <Icon icon="ph:spinner" class="animate-spin" /> Loading packages...
      </p>
    {:else if packages.length === 0}
      <CloudEmptyState
        icon="ph:package"
        heading="No local packages yet"
        message="Right-click a project with a typst.toml in your workspace and choose Install as @local package."
      />
    {:else if visiblePackages.length === 0}
      <div class="empty-state py-16">
        <Icon icon="ph:magnifying-glass-minus" class="text-3xl" />
        <p class="text-sm">No packages match your filter.</p>
      </div>
    {:else}
      <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
        {#each visiblePackages as pack (pack.name)}
          <PackageCard {pack} onremove={(version) => (pendingRemoval = { name: pack.name, version })} />
        {/each}
      </div>
    {/if}
  </div>
</div>

{#if pendingRemoval}
  <ConfirmModal
    title="Remove package"
    message="Remove @local/{pendingRemoval.name}:{pendingRemoval.version}? Documents that import this version will no longer compile."
    confirmLabel="Remove"
    onconfirm={confirmRemoval}
    onclose={() => (pendingRemoval = null)}
  />
{/if}
