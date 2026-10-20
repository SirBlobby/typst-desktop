<script lang="ts">
  import Icon from "@iconify/svelte";
  import * as api from "$lib/ts/api";
  import type { BrowseEntry, EntryKind } from "$lib/ts/api";
  import { clampMenu } from "$lib/ts/menu-position";
  import {
    app,
    breadcrumbs,
    browseTo,
    openTarget,
    refreshEntries,
    setError,
  } from "$lib/ts/state.svelte";

  interface Props {
    onnewfolder: () => void;
    onnewproject: () => void;
    onnewdocument: () => void;
    onupload: () => void;
    onrename: (entry: BrowseEntry) => void;
    ondelete: (entry: BrowseEntry) => void;
    onlink: (entry: BrowseEntry) => void;
    onsavetocloud: (entry: BrowseEntry) => void;
    onviewimage: (paths: string[], index: number) => void;
  }

  let {
    onnewfolder,
    onnewproject,
    onnewdocument,
    onupload,
    onrename,
    ondelete,
    onlink,
    onsavetocloud,
    onviewimage,
  }: Props = $props();

  let menuFor = $state<string | null>(null);
  let menuAt = $state({ x: 0, y: 0 });

  function openContextMenu(event: MouseEvent, path: string) {
    event.preventDefault();
    event.stopPropagation();
    menuAt = { x: event.clientX, y: event.clientY };
    menuFor = path;
  }

  const trail = $derived(breadcrumbs());
  let newMenuOpen = $state(false);

  const headerTitle = $derived(trail.at(-1)?.name ?? "Workspace");

  const containers = $derived(
    app.entries.filter(
      (entry) => entry.kind === "folder" || entry.kind === "project",
    ),
  );

  const documents = $derived(
    app.entries.filter(
      (entry) => entry.kind === "document" || entry.kind === "file",
    ),
  );

  let thumbs = $state<Record<string, { kind: string; data: string }>>({});

  $effect(() => {
    const pending = documents.map((entry) => entry.path);
    let cancelled = false;

    (async () => {
      for (const path of pending) {
        if (cancelled) return;
        if (thumbs[path]) continue;
        try {
          const result = await api.thumbnail(path);
          if (!cancelled) thumbs[path] = result;
        } catch {
          continue;
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  });

  const iconFor: Record<EntryKind, string> = {
    project: "ph:folder-star",
    folder: "ph:folder",
    document: "ph:file-text",
    file: "ph:file",
  };

  const colorFor: Record<EntryKind, string> = {
    project: "text-[var(--color-accent)]",
    folder: "text-[var(--color-ink-muted)]",
    document: "text-[var(--color-accent)]",
    file: "text-[var(--color-ink-muted)]",
  };

  function relativeTime(value: string | null): string {
    if (!value) return "";
    const then = new Date(value).getTime();
    if (Number.isNaN(then)) return "";

    const minutes = Math.round((Date.now() - then) / 60000);
    if (minutes < 1) return "just now";
    if (minutes < 60) return `${minutes}m ago`;
    if (minutes < 1440) return `${Math.round(minutes / 60)}h ago`;
    return `${Math.round(minutes / 1440)}d ago`;
  }

  function syncLabel(entry: BrowseEntry): string {
    if (entry.sync_state === "pending") return "Local changes not yet synced";
    if (entry.sync_state === "synced") {
      const when = relativeTime(entry.last_synced_at);
      return when ? `Synced ${when}` : "Synced";
    }
    return "Linked to the cloud";
  }

  const imagePaths = $derived(
    app.entries
      .filter((entry) => api.isImagePath(entry.path))
      .map((entry) => entry.path),
  );

  let dragging = $state<string | null>(null);
  let dropTarget = $state<string | null>(null);

  async function moveTo(source: string, destination: string) {
    if (source === destination) return;
    const parent = source.includes("/") ? source.slice(0, source.lastIndexOf("/")) : "";
    if (parent === destination) return;
    try {
      await api.moveEntry(source, destination);
      await refreshEntries();
    } catch (error) {
      setError(error);
    }
  }

  function startDrag(event: DragEvent, path: string) {
    dragging = path;
    event.dataTransfer?.setData("text/plain", path);
  }

  function endDrag() {
    dragging = null;
    dropTarget = null;
  }

  function allowDrop(event: DragEvent, path: string) {
    if (!dragging) return;
    event.preventDefault();
    dropTarget = path;
  }

  function handleDrop(event: DragEvent, destination: string) {
    event.preventDefault();
    const source = dragging ?? event.dataTransfer?.getData("text/plain");
    dragging = null;
    dropTarget = null;
    if (source) moveTo(source, destination);
  }

  function activate(entry: BrowseEntry) {
    if (entry.kind === "folder") {
      browseTo(entry.path);
    } else if (entry.kind === "project" || entry.kind === "document") {
      openTarget(entry.path);
    } else if (api.isImagePath(entry.path)) {
      onviewimage(imagePaths, imagePaths.indexOf(entry.path));
    }
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  }

  function formatDate(value: string | null): string {
    if (!value) return "";
    const date = new Date(value);
    return Number.isNaN(date.getTime()) ? "" : date.toLocaleDateString();
  }
</script>

{#snippet sectionTitle(label: string)}
  <div class="mb-2 flex items-center gap-2">
    <h2 class="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">{label}</h2>
    <div class="h-px flex-1 bg-[var(--color-line)]"></div>
  </div>
{/snippet}

{#snippet syncBadge(entry: BrowseEntry)}
  {#if entry.cloud_linked}
    <span class="flex shrink-0 items-center" title={syncLabel(entry)}>
      {#if entry.sync_state === "pending"}
        <Icon
          icon="ph:cloud-arrow-up"
          class="text-sm text-[var(--color-accent)]"
        />
      {:else}
        <Icon
          icon="ph:cloud-check"
          class="text-sm text-[var(--color-success)]"
        />
      {/if}
    </span>
  {/if}
{/snippet}

{#snippet actions(entry: BrowseEntry)}
  <button
    data-card-menu
    class="shrink-0 rounded p-1 text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]"
    onclick={(event) => {
      event.stopPropagation();
      if (menuFor === entry.path) {
        menuFor = null;
        return;
      }
      const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
      menuAt = { x: rect.right, y: rect.bottom + 4 };
      menuFor = entry.path;
    }}
    aria-label="Actions"
  >
    <Icon icon="ph:dots-three-vertical" />
  </button>

  {#if menuFor === entry.path}
    <div
      use:clampMenu
      class="fixed z-50 flex w-40 flex-col rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] py-1 text-xs shadow-lg"
      style="left: {menuAt.x}px; top: {menuAt.y}px"
    >
      {#if entry.kind === "project" || entry.kind === "document"}
        <button
          class="px-3 py-1.5 text-left hover:bg-[var(--color-surface-sunken)]"
          onclick={() => {
            openTarget(entry.path);
            menuFor = null;
          }}
        >
          Open in editor
        </button>
      {:else if api.isImagePath(entry.path)}
        <button
          class="px-3 py-1.5 text-left hover:bg-[var(--color-surface-sunken)]"
          onclick={() => {
            onviewimage(imagePaths, imagePaths.indexOf(entry.path));
            menuFor = null;
          }}
        >
          Open image
        </button>
      {/if}
      {#if entry.kind === "project" && !entry.cloud_project_id && app.account}
        <button
          class="px-3 py-1.5 text-left hover:bg-[var(--color-surface-sunken)]"
          onclick={() => {
            onlink(entry);
            menuFor = null;
          }}
        >
          Upload to cloud
        </button>
      {/if}
      {#if entry.kind === "document" && !entry.cloud_linked && app.account}
        <button
          class="px-3 py-1.5 text-left hover:bg-[var(--color-surface-sunken)]"
          onclick={() => {
            onsavetocloud(entry);
            menuFor = null;
          }}
        >
          Save to cloud
        </button>
      {/if}
      <button
        class="px-3 py-1.5 text-left hover:bg-[var(--color-surface-sunken)]"
        onclick={() => {
          onrename(entry);
          menuFor = null;
        }}
      >
        Rename
      </button>
      <button
        class="px-3 py-1.5 text-left text-[var(--color-danger)] hover:bg-[var(--color-surface-sunken)]"
        onclick={() => {
          ondelete(entry);
          menuFor = null;
        }}
      >
        Delete
      </button>
    </div>
  {/if}
{/snippet}

<svelte:window
  on:click={(event) => {
    if (!(event.target as HTMLElement).closest("[data-card-menu]")) {
      menuFor = null;
    }
    if (!(event.target as HTMLElement).closest("[data-new-menu]")) {
      newMenuOpen = false;
    }
  }}
/>

<div
  class="flex h-full flex-col bg-[var(--color-surface-muted)]"
  role="presentation"
  oncontextmenu={(event) => event.preventDefault()}
>
  <header class="shrink-0 border-b border-[var(--color-line)] bg-[var(--color-surface)] px-5 pb-3 pt-3">
    <nav class="mb-1 flex flex-wrap items-center gap-0.5 text-[11px] text-[var(--color-ink-muted)]" aria-label="Location">
      <button
        class="flex items-center gap-1 rounded px-1.5 py-0.5 transition hover:text-[var(--color-ink)]
          {dropTarget === '' ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]' : ''}"
        onclick={() => browseTo("")}
        ondragover={(event) => allowDrop(event, "")}
        ondragleave={() => {
          if (dropTarget === "") dropTarget = null;
        }}
        ondrop={(event) => handleDrop(event, "")}
      >
        <Icon icon="ph:hard-drives" />
        Workspace
      </button>
      {#each trail as crumb}
        <span class="opacity-50">/</span>
        <button
          class="max-w-48 truncate rounded px-1.5 py-0.5 transition hover:text-[var(--color-ink)]
            {dropTarget === crumb.path ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]' : ''}"
          onclick={() => browseTo(crumb.path)}
          ondragover={(event) => allowDrop(event, crumb.path)}
          ondragleave={() => {
            if (dropTarget === crumb.path) dropTarget = null;
          }}
          ondrop={(event) => handleDrop(event, crumb.path)}
        >
          {crumb.name}
        </button>
      {/each}
    </nav>

    <div class="flex items-center gap-3">
      <h1 class="min-w-0 flex-1 truncate text-lg font-semibold">{headerTitle}</h1>

      <button
        class="flex items-center gap-1.5 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-2.5 py-1.5 text-xs font-medium transition hover:bg-[var(--color-surface-sunken)]"
        onclick={onupload}
      >
        <Icon icon="ph:upload-simple" />
        Import
      </button>

      <div class="relative flex" data-new-menu>
        <button
          class="flex items-center gap-1.5 rounded-l-md bg-[var(--color-accent)] py-1.5 pl-2.5 pr-2 text-xs font-semibold text-white transition hover:opacity-90"
          onclick={onnewdocument}
        >
          <Icon icon="ph:file-plus" />
          New document
        </button>
        <button
          class="rounded-r-md border-l border-white/25 bg-[var(--color-accent)] px-1.5 py-1.5 text-xs text-white transition hover:opacity-90"
          aria-label="More create options"
          aria-haspopup="menu"
          aria-expanded={newMenuOpen}
          onclick={() => (newMenuOpen = !newMenuOpen)}
        >
          <Icon icon="ph:caret-down" />
        </button>
        {#if newMenuOpen}
          <div role="menu" class="absolute right-0 top-full z-30 mt-1 w-52 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] p-1 shadow-xl">
            {#each [
              { label: "Project", description: "Multi-file workspace", icon: "ph:folder-star", action: onnewproject },
              { label: "Folder", description: "Organize your files", icon: "ph:folder-plus", action: onnewfolder },
            ] as item}
              <button
                role="menuitem"
                class="flex w-full items-center gap-2.5 rounded-md px-2.5 py-2 text-left hover:bg-[var(--color-surface-sunken)]"
                onclick={() => {
                  newMenuOpen = false;
                  item.action();
                }}
              >
                <Icon icon={item.icon} class="text-lg text-[var(--color-accent)]" />
                <span>
                  <span class="block text-xs font-medium">{item.label}</span>
                  <span class="block text-[10px] text-[var(--color-ink-muted)]">{item.description}</span>
                </span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </header>

  <div class="scroll-thin flex-1 overflow-y-auto p-4">
    {#if app.entries.length === 0}
      <div
        class="flex h-full flex-col items-center justify-center gap-3 text-[var(--color-ink-muted)]"
      >
        <Icon icon="ph:folder-open" class="text-5xl" />
        <p class="text-sm">This folder is empty.</p>
        <div class="flex gap-2">
          <button
            class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white hover:opacity-90"
            onclick={onnewdocument}
          >
            New document
          </button>
          <button
            class="rounded-md border border-[var(--color-line)] px-3 py-1.5 text-xs hover:bg-[var(--color-surface)]"
            onclick={onnewproject}
          >
            New project
          </button>
        </div>
      </div>
    {:else}
      <div class="flex flex-col gap-5">
        {#if containers.length > 0}
          <section class="flex flex-col gap-2">
            {@render sectionTitle("Folders")}
            <div class="grid grid-cols-[repeat(auto-fill,minmax(210px,1fr))] gap-2">
              {#each containers as entry (entry.path)}
                <div
                  class="group relative flex items-center gap-2.5 rounded-lg border px-3 py-2.5 transition hover:border-[var(--color-accent)] hover:bg-[var(--color-surface)]
                    {dropTarget === entry.path
                    ? 'border-[var(--color-accent)] bg-[var(--color-accent-soft)]'
                    : 'border-[var(--color-line)] bg-[var(--color-surface-sunken)]'}"
                  role="group"
                  draggable="true"
                  ondragstart={(event) => startDrag(event, entry.path)}
                  ondragend={endDrag}
                  ondragover={(event) => allowDrop(event, entry.path)}
                  ondragleave={() => {
                    if (dropTarget === entry.path) dropTarget = null;
                  }}
                  ondrop={(event) => handleDrop(event, entry.path)}
                  oncontextmenu={(event) => openContextMenu(event, entry.path)}
                >
                  <button
                    class="flex min-w-0 flex-1 items-center gap-2.5 text-left"
                    ondblclick={() => activate(entry)}
                    onclick={() => activate(entry)}
                  >
                    <Icon
                      icon={entry.kind === "project"
                        ? "ph:folder-star-fill"
                        : "ph:folder-fill"}
                      class="shrink-0 text-2xl {entry.kind === 'project'
                        ? 'text-[var(--color-accent)]'
                        : 'text-[var(--color-ink-muted)]'}"
                    />
                    <span class="min-w-0 flex-1">
                      <span class="flex items-center gap-1.5">
                        <span
                          class="truncate text-xs font-medium"
                          title={entry.name}
                        >
                          {entry.name}
                        </span>
                        {@render syncBadge(entry)}
                      </span>
                      <span
                        class="block truncate text-[10px] text-[var(--color-ink-muted)]"
                      >
                        {entry.kind === "project" ? "Project · " : ""}{entry.child_count}
                        items
                      </span>
                    </span>
                  </button>

                  {@render actions(entry)}
                </div>
              {/each}
            </div>
          </section>
        {/if}

        {#if documents.length > 0}
          <section class="flex flex-col gap-2">
            {@render sectionTitle("Documents")}
            <div class="grid grid-cols-[repeat(auto-fill,minmax(150px,1fr))] gap-3">
              {#each documents as entry (entry.path)}
                <div
                  class="group relative flex flex-col overflow-hidden rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] transition hover:border-[var(--color-accent)] hover:shadow-md"
                  role="group"
                  draggable="true"
                  ondragstart={(event) => startDrag(event, entry.path)}
                  ondragend={endDrag}
                  oncontextmenu={(event) => openContextMenu(event, entry.path)}
                >
                  <button
                    class="flex flex-col text-left"
                    ondblclick={() => activate(entry)}
                    onclick={() => activate(entry)}
                  >
                    <span
                      class="flex h-24 items-center justify-center overflow-hidden border-b border-[var(--color-line)] bg-[var(--color-surface-muted)]"
                    >
                      {#if thumbs[entry.path]?.kind === "svg"}
                        <span
                          class="flex w-full origin-top scale-100 items-start justify-center bg-white p-1 [&_svg]:h-auto [&_svg]:w-full"
                        >
                          {@html thumbs[entry.path].data}
                        </span>
                      {:else if thumbs[entry.path]?.kind === "image"}
                        <img
                          src={thumbs[entry.path].data}
                          alt={entry.name}
                          class="h-full w-full object-contain"
                        />
                      {:else}
                        <Icon
                          icon={iconFor[entry.kind]}
                          class="text-3xl {colorFor[entry.kind]}"
                        />
                      {/if}
                    </span>

                  </button>

                  <div class="flex items-center gap-1 px-2.5 py-2">
                    <button
                      class="flex min-w-0 flex-1 flex-col gap-0.5 text-left"
                      onclick={() => activate(entry)}
                    >
                      <span class="flex items-center gap-1.5">
                        <span
                          class="min-w-0 flex-1 truncate text-xs font-medium"
                          title={entry.name}
                        >
                          {entry.name}
                        </span>
                        {@render syncBadge(entry)}
                      </span>
                      <span class="text-[10px] text-[var(--color-ink-muted)]">
                        {#if entry.cloud_linked && entry.sync_state === "pending"}
                          Not synced
                        {:else if entry.cloud_linked}
                          Synced {relativeTime(entry.last_synced_at)}
                        {:else}
                          {formatSize(entry.size)}
                          {#if entry.modified}
                            · {formatDate(entry.modified)}
                          {/if}
                        {/if}
                      </span>
                    </button>

                    {@render actions(entry)}
                  </div>
                </div>
              {/each}
            </div>
          </section>
        {/if}
      </div>
    {/if}
  </div>
</div>
