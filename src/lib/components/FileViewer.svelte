<script lang="ts">
  import Icon from "@iconify/svelte";
  import * as api from "#lib/ts/api";
  import type { BrowseEntry } from "#lib/ts/api";
  import { clampMenu } from "#lib/ts/menu-position";
  import PageThumbnail from "./PageThumbnail.svelte";
  import FolderColorSwatches from "./FolderColorSwatches.svelte";
  import FilterBar from "./FilterBar.svelte";
  import LocalHeader from "./local/LocalHeader.svelte";
  import LocalListView from "./local/LocalListView.svelte";
  import LocalSyncBadge from "./local/LocalSyncBadge.svelte";
  import { folderIconStyle } from "#lib/ts/folder-colors";
  import { entryIcon, formatDate, formatSize, isContainer, relativeTime } from "#lib/ts/local-entries";
  import {
    allSortOptions,
    defaultViewFilters,
    matchesQuery,
    readStoredPreference,
    sortByFilters,
    writeStoredPreference,
    type ViewFilters,
    type ViewMode,
  } from "#lib/ts/view-filters";
  import {
    app,
    breadcrumbs,
    browseTo,
    openTarget,
    refreshEntries,
    setError,
    shortcutForPath,
  } from "#lib/ts/state.svelte";

  interface Props {
    onnewfolder: () => void;
    onnewproject: () => void;
    onnewdocument: () => void;
    onupload: () => void;
    onconvert: () => void;
    onrename: (entry: BrowseEntry) => void;
    ondelete: (entry: BrowseEntry) => void;
    onlink: (entry: BrowseEntry) => void;
    onsavetocloud: (entry: BrowseEntry) => void;
    onviewfile: (paths: string[], index: number) => void;
  }

  let {
    onnewfolder,
    onnewproject,
    onnewdocument,
    onupload,
    onconvert,
    onrename,
    ondelete,
    onlink,
    onsavetocloud,
    onviewfile,
  }: Props = $props();

  const FILTERS_KEY = "typst-desktop-local-filters";
  const VIEW_MODE_KEY = "typst-desktop-local-view-mode";

  let menuFor = $state<string | null>(null);
  let menuAt = $state({ x: 0, y: 0 });
  let searchQuery = $state("");
  let filters = $state<ViewFilters>(readStoredPreference(FILTERS_KEY, defaultViewFilters));
  let viewMode = $state<ViewMode>(readStoredPreference(VIEW_MODE_KEY, "grid"));

  $effect(() => writeStoredPreference(FILTERS_KEY, filters));
  $effect(() => writeStoredPreference(VIEW_MODE_KEY, viewMode));

  $effect(() => {
    app.currentDir;
    searchQuery = "";
  });

  function openContextMenu(event: MouseEvent, path: string) {
    event.preventDefault();
    event.stopPropagation();
    menuAt = { x: event.clientX, y: event.clientY };
    menuFor = path;
  }

  const trail = $derived(breadcrumbs());
  const headerTitle = $derived(trail.at(-1)?.name ?? "Workspace");
  const ancestors = $derived(trail.slice(0, -1));
  const inShortcut = $derived(shortcutForPath(app.currentDir) !== null);

  const visibleEntries = $derived(
    sortByFilters(
      app.entries.filter((entry) => matchesQuery(entry.name, searchQuery)),
      filters,
      (entry) => entry,
    ),
  );
  const visibleContainers = $derived(visibleEntries.filter(isContainer));
  const visibleDocuments = $derived(visibleEntries.filter((entry) => !isContainer(entry)));

  let thumbs = $state<Record<string, { kind: string; data: string }>>({});

  $effect(() => {
    const pending = app.entries.filter((entry) => !isContainer(entry)).map((entry) => entry.path);
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

  const viewablePaths = $derived(
    visibleEntries
      .filter((entry) => entry.kind === "file" && api.isViewablePath(entry.path))
      .map((entry) => entry.path),
  );

  function viewFile(path: string) {
    onviewfile(viewablePaths, viewablePaths.indexOf(path));
  }

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

  function leaveDrop(path: string) {
    if (dropTarget === path) dropTarget = null;
  }

  function handleDrop(event: DragEvent, destination: string) {
    event.preventDefault();
    const source = dragging ?? event.dataTransfer?.getData("text/plain");
    dragging = null;
    dropTarget = null;
    if (source) moveTo(source, destination);
  }

  async function changeFolderColor(entry: BrowseEntry, color: string | null) {
    menuFor = null;
    if (color === entry.color) return;
    try {
      await api.setFolderColor(entry.path, color);
      await refreshEntries();
    } catch (error) {
      setError(error);
    }
  }

  function activate(entry: BrowseEntry) {
    if (entry.kind === "folder") {
      browseTo(entry.path);
    } else if (entry.kind === "project" || entry.kind === "document") {
      openTarget(entry.path);
    } else if (api.isViewablePath(entry.path)) {
      viewFile(entry.path);
    }
  }
</script>

{#snippet sectionTitle(label: string, count: number)}
  <h2 class="section-label flex items-center gap-1.5">
    {label}
    <span class="tabular-nums text-[var(--color-ink-subtle)]/70">{count}</span>
  </h2>
{/snippet}

{#snippet actions(entry: BrowseEntry)}
  <button
    data-card-menu
    class="icon-btn icon-btn-sm opacity-0 group-hover:opacity-100 focus-visible:opacity-100 {menuFor === entry.path ? 'opacity-100' : ''}"
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
    <Icon icon="ph:dots-three" class="text-base" />
  </button>

  {#if menuFor === entry.path}
    <div
      use:clampMenu
      class="menu fixed z-50 w-48"
      style="left: {menuAt.x}px; top: {menuAt.y}px"
    >
      {#if entry.kind === "folder"}
        <FolderColorSwatches color={entry.color} onchoose={(color) => changeFolderColor(entry, color)} />
      {/if}
      {#if entry.kind === "project" || entry.kind === "document"}
        <button
          class="menu-item"
          onclick={() => {
            openTarget(entry.path);
            menuFor = null;
          }}
        >
          Open in editor
        </button>
      {:else if api.isViewablePath(entry.path)}
        <button
          class="menu-item"
          onclick={() => {
            viewFile(entry.path);
            menuFor = null;
          }}
        >
          {api.isPdfPath(entry.path) ? "Open PDF" : "Open image"}
        </button>
      {/if}
      {#if entry.kind === "project" && !entry.cloud_project_id && app.account}
        <button
          class="menu-item"
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
          class="menu-item"
          onclick={() => {
            onsavetocloud(entry);
            menuFor = null;
          }}
        >
          Save to cloud
        </button>
      {/if}
      <button
        class="menu-item"
        onclick={() => {
          onrename(entry);
          menuFor = null;
        }}
      >
        Rename
      </button>
      <button
        class="menu-item menu-item-danger"
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
  onclick={(event) => {
    if (!(event.target as HTMLElement).closest("[data-card-menu]")) {
      menuFor = null;
    }
  }}
/>

<div
  class="flex h-full flex-col bg-[var(--color-surface-muted)]"
  role="presentation"
  oncontextmenu={(event) => event.preventDefault()}
>
  <LocalHeader
    title={headerTitle}
    {ancestors}
    showTrail={trail.length > 0 && (!inShortcut || ancestors.length > 0)}
    {inShortcut}
    {dropTarget}
    bind:searchQuery
    ondragovercrumb={allowDrop}
    ondragleavecrumb={leaveDrop}
    ondropcrumb={handleDrop}
    {onupload}
    {onnewdocument}
    {onnewproject}
    {onnewfolder}
    {onconvert}
  />

  <div class="scroll-thin flex-1 overflow-y-auto px-6 pb-6">
    {#if app.entries.length === 0}
      <div class="empty-state h-full gap-3">
        <span class="flex h-14 w-14 items-center justify-center rounded-2xl bg-[var(--color-surface-sunken)]">
          <Icon icon="ph:folder-open" class="text-3xl text-[var(--color-ink-muted)]" />
        </span>
        <p class="text-sm text-[var(--color-ink-muted)]">This folder is empty</p>
        <div class="flex gap-2">
          <button class="btn btn-primary" onclick={onnewdocument}>New document</button>
          <button class="btn btn-secondary" onclick={onnewproject}>New project</button>
        </div>
      </div>
    {:else}
      <FilterBar bind:filters bind:viewMode sortOptions={allSortOptions} />

      {#if visibleEntries.length === 0}
        <div class="empty-state py-16">
          <Icon icon="ph:magnifying-glass-minus" class="text-3xl" />
          <p class="text-sm">Nothing here matches your search.</p>
        </div>
      {:else if viewMode === "list"}
        <LocalListView
          entries={visibleEntries}
          {dropTarget}
          {actions}
          onactivate={activate}
          oncontextmenu={openContextMenu}
          ondragstart={startDrag}
          ondragend={endDrag}
          ondragover={allowDrop}
          ondragleave={leaveDrop}
          ondrop={handleDrop}
        />
      {:else}
        <div class="flex flex-col gap-6">
          {#if visibleContainers.length > 0}
            <section class="flex flex-col gap-2.5">
              {@render sectionTitle("Folders", visibleContainers.length)}
              <div class="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-2.5">
                {#each visibleContainers as entry (entry.path)}
                  <div
                    class="card card-interactive group relative flex items-center gap-2.5 py-2.5 pl-3 pr-2
                      {dropTarget === entry.path
                      ? 'border-[var(--color-accent)] bg-[var(--color-accent-soft)]'
                      : ''}"
                    role="group"
                    draggable="true"
                    ondragstart={(event) => startDrag(event, entry.path)}
                    ondragend={endDrag}
                    ondragover={(event) => allowDrop(event, entry.path)}
                    ondragleave={() => leaveDrop(entry.path)}
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
                        class="shrink-0 text-[1.375rem] {entry.kind === 'project' ? 'text-[var(--color-accent)]' : ''}"
                        style={entry.kind === "folder" ? folderIconStyle(entry.color) : undefined}
                      />
                      <span class="min-w-0 flex-1">
                        <span class="flex items-center gap-1.5">
                          <span
                            class="truncate text-[13px] font-medium"
                            title={entry.name}
                          >
                            {entry.name}
                          </span>
                          <LocalSyncBadge {entry} />
                        </span>
                        <span
                          class="block truncate text-[11px] text-[var(--color-ink-muted)]"
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

          {#if visibleDocuments.length > 0}
            <section class="flex flex-col gap-2.5">
              {@render sectionTitle("Documents", visibleDocuments.length)}
              <div class="grid grid-cols-[repeat(auto-fill,minmax(160px,1fr))] gap-3">
                {#each visibleDocuments as entry (entry.path)}
                  <div
                    class="card card-interactive group relative flex flex-col overflow-hidden"
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
                      <PageThumbnail
                        thumbnail={thumbs[entry.path] ?? null}
                        name={entry.name}
                        fallbackIcon={entryIcon(entry)}
                        fallbackClass={entry.kind === "document"
                          ? "bg-[var(--color-accent-soft)] text-[var(--color-accent)]"
                          : "bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]"}
                      />
                    </button>

                    <div class="flex items-center gap-1 py-2 pl-3 pr-1.5">
                      <button
                        class="flex min-w-0 flex-1 flex-col gap-0.5 text-left"
                        onclick={() => activate(entry)}
                      >
                        <span class="flex items-center gap-1.5">
                          <span
                            class="min-w-0 flex-1 truncate text-[13px] font-medium"
                            title={entry.name}
                          >
                            {entry.name}
                          </span>
                          <LocalSyncBadge {entry} />
                        </span>
                        <span class="text-[11px] text-[var(--color-ink-muted)]">
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
    {/if}
  </div>
</div>
