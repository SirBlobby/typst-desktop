<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import Icon from "@iconify/svelte";
  import ConfirmModal from "$lib/components/ConfirmModal.svelte";
  import PromptModal from "$lib/components/PromptModal.svelte";
  import VersionTimeline from "./VersionTimeline.svelte";
  import VersionDiffView from "./VersionDiffView.svelte";
  import PersonAvatar from "./PersonAvatar.svelte";
  import { reviewSessionStore } from "$lib/ts/review/session";
  import { diffLines, summarize } from "$lib/ts/review/line-diff";
  import {
    fetchVersions,
    renameVersion,
    restoreVersion,
    versionDay,
    versionKindLabels,
    versionTime,
    type DocumentVersion,
  } from "$lib/ts/review/versions";

  type CompareTarget = "previous" | "current";

  let { title, onClose }: { title: string; onClose: () => void } = $props();

  const session = get(reviewSessionStore)!;
  const canEdit = session.role === "owner" || session.role === "editor";

  let versions = $state<DocumentVersion[]>([]);
  let selected = $state<DocumentVersion | null>(null);
  let compareTarget = $state<CompareTarget>("previous");
  let showUnchanged = $state(false);
  let loading = $state(true);
  let errorMessage = $state("");
  let confirmRestore = $state(false);
  let renaming = $state(false);
  let busy = $state(false);

  let previousVersion = $derived.by(() => {
    if (!selected) return null;
    const index = versions.findIndex((version) => version.id === selected!.id);
    return index >= 0 ? (versions[index + 1] ?? null) : null;
  });

  let diff = $derived.by(() => {
    if (!selected) return [];
    if (compareTarget === "current") return diffLines(selected.content, session.text.toString());
    return diffLines(previousVersion?.content ?? "", selected.content);
  });

  let changeSummary = $derived(summarize(diff));

  async function load() {
    loading = true;
    errorMessage = "";
    try {
      versions = await fetchVersions(session.documentId);
      selected = versions.find((version) => version.id === selected?.id) ?? versions[0] ?? null;
    } catch (error) {
      errorMessage = String(error);
    }
    loading = false;
  }

  async function submitRename(label: string) {
    renaming = false;
    if (!selected) return;
    try {
      await renameVersion(session.documentId, selected.id, label);
      await load();
    } catch (error) {
      errorMessage = String(error);
    }
  }

  async function restoreSelected() {
    confirmRestore = false;
    if (!selected) return;
    busy = true;
    try {
      await restoreVersion(session.documentId, selected.id);
      onClose();
    } catch (error) {
      errorMessage = String(error);
    }
    busy = false;
  }

  function handleKey(event: KeyboardEvent) {
    if (event.key === "Escape" && !confirmRestore && !renaming) onClose();
  }

  onMount(load);
</script>

<svelte:window onkeydown={handleKey} />

<div class="fixed inset-0 z-50 flex flex-col bg-[var(--color-surface-muted)]">
  <header class="flex items-center gap-3 border-b border-[var(--color-line)] bg-[var(--color-surface)] px-4 py-2">
    <button
      onclick={onClose}
      class="rounded p-1 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]"
      title="Back to the editor"
      aria-label="Back to the editor"
    >
      <Icon icon="ph:arrow-left" class="text-lg" />
    </button>
    <div class="min-w-0">
      <p class="text-[10px] uppercase tracking-wider text-[var(--color-ink-muted)]">Version history</p>
      <h1 class="truncate text-sm font-semibold">{title}</h1>
    </div>
    <div class="ml-auto flex items-center gap-2">
      {#if selected && canEdit}
        <button
          onclick={() => (renaming = true)}
          class="flex items-center gap-1.5 rounded-md border border-[var(--color-line)] px-3 py-1.5 text-xs font-medium hover:bg-[var(--color-surface-sunken)]"
        >
          <Icon icon="mdi:tag-outline" class="text-sm" />
          {selected.label ? "Rename" : "Name this version"}
        </button>
        <button
          onclick={() => (confirmRestore = true)}
          disabled={busy}
          class="flex items-center gap-1.5 rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-semibold text-white hover:opacity-90 disabled:opacity-50"
        >
          <Icon icon={busy ? "mdi:loading" : "mdi:restore"} class="text-sm {busy ? 'animate-spin' : ''}" />
          Restore this version
        </button>
      {/if}
    </div>
  </header>

  {#if errorMessage}
    <div class="border-b border-[var(--color-danger)]/30 bg-[var(--color-danger)]/10 px-4 py-2 text-xs text-[var(--color-danger)]">{errorMessage}</div>
  {/if}

  <div class="flex min-h-0 flex-1">
    <main class="scroll-thin min-h-0 flex-1 overflow-y-auto p-5">
      {#if loading}
        <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading history...</p>
      {:else if !selected}
        <div class="flex h-full flex-col items-center justify-center gap-2 text-center text-[var(--color-ink-muted)]">
          <Icon icon="mdi:history" class="text-5xl opacity-50" />
          <p class="text-sm font-medium text-[var(--color-ink)]">No versions yet</p>
          <p class="max-w-sm text-xs">A version is saved every time someone leaves the editor after making changes, or when you save a version.</p>
        </div>
      {:else}
        <div class="mb-4 flex flex-wrap items-center gap-3">
          <div class="flex -space-x-1.5">
            {#each selected.editors as editor (editor.id)}
              <PersonAvatar person={editor} sizeClass="h-8 w-8" textClass="text-xs" />
            {/each}
          </div>
          <div class="min-w-0">
            <p class="text-sm font-semibold">{selected.label ?? `${versionKindLabels[selected.kind]} version`}</p>
            <p class="text-xs text-[var(--color-ink-muted)]">
              {versionDay(selected)} at {versionTime(selected)} · Edited by {selected.editors.map((editor) => editor.name).join(", ") || "unknown"}
            </p>
          </div>
          <div class="ml-auto flex items-center gap-3">
            <span class="text-xs font-semibold text-emerald-600">+{changeSummary.added}</span>
            <span class="text-xs font-semibold text-rose-600">-{changeSummary.removed}</span>
            <select
              bind:value={compareTarget}
              aria-label="Compare with"
              class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-2 py-1 text-xs"
            >
              <option value="previous">Changes in this version</option>
              <option value="current">Compare with current document</option>
            </select>
            <label class="flex items-center gap-1.5 text-xs text-[var(--color-ink-muted)]">
              <input type="checkbox" bind:checked={showUnchanged} />
              Show all lines
            </label>
          </div>
        </div>
        <VersionDiffView lines={diff} {showUnchanged} />
      {/if}
    </main>

    <aside class="w-80 shrink-0 border-l border-[var(--color-line)] bg-[var(--color-surface)]">
      <VersionTimeline {versions} selectedId={selected?.id ?? null} onSelect={(version) => (selected = version)} />
    </aside>
  </div>
</div>

{#if confirmRestore && selected}
  <ConfirmModal
    title="Restore this version"
    message="The document will be replaced with this version for everyone. The current content is saved as a version first, so you can undo this from history."
    confirmLabel="Restore"
    onconfirm={restoreSelected}
    onclose={() => (confirmRestore = false)}
  />
{/if}

{#if renaming && selected}
  <PromptModal
    title="Name this version"
    label="Version name"
    icon="mdi:tag-outline"
    value={selected.label ?? ""}
    confirmLabel="Save"
    onsubmit={submitRename}
    onclose={() => (renaming = false)}
  />
{/if}
