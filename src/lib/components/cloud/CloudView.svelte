<script lang="ts">
  import { untrack } from "svelte";
  import Icon from "@iconify/svelte";
  import * as api from "#lib/ts/api";
  import type { CloudFile, CloudFolder } from "#lib/ts/api";
  import {
    cloudSortOptions,
    defaultCloudFilters,
    documentEntry,
    fileEntry,
    folderEntry,
    matchesSearch,
    projectEntry,
    sortEntries,
    type CloudEntry,
    type CloudFilters,
    type CloudHandlers,
    type ViewMode,
  } from "#lib/ts/cloud-entries";
  import {
    app,
    cloudBreadcrumbs,
    linkedDocument,
    linkedProject,
    openCloudFolder,
    openTarget,
    refreshCloud,
    setError,
  } from "#lib/ts/state.svelte";
  import { activeCloudSpace, canChangeActiveSpace } from "#lib/ts/cloud-spaces";
  import { readStoredPreference, writeStoredPreference } from "#lib/ts/view-filters";
  import CloudActionMenu from "./CloudActionMenu.svelte";
  import CloudEmptyState from "./CloudEmptyState.svelte";
  import FilterBar from "../FilterBar.svelte";
  import CloudHeader from "./CloudHeader.svelte";
  import CloudListView from "./CloudListView.svelte";
  import CloudRowCard from "./CloudRowCard.svelte";
  import CloudSectionTitle from "./CloudSectionTitle.svelte";
  import CloudTileCard from "./CloudTileCard.svelte";

  interface Props {
    ondownloaddocument: (documentId: string, title: string) => void;
    onremovedownload: (path: string) => void;
    ondownloadfile: (fileId: string, name: string) => void;
    onopenfile: (fileId: string, name: string) => void;
    ondeletefile: (fileId: string) => void;
    onuploadfile: () => void;
    onrenamefile: (file: CloudFile) => void;
    oncloneproject: (cloudProjectId: string, name: string) => void;
    ondeleteproject: (cloudProjectId: string) => void;
    ondeletedocument: (documentId: string) => void;
    onnewcloudproject: () => void;
    onnewclouddocument: () => void;
    onnewcloudfolder: () => void;
    onrenamecloudfolder: (folder: CloudFolder) => void;
    ondeletecloudfolder: (folder: CloudFolder) => void;
    onsignin: () => void;
  }

  let {
    ondownloaddocument,
    onremovedownload,
    ondownloadfile,
    onopenfile,
    ondeletefile,
    onuploadfile,
    onrenamefile,
    oncloneproject,
    ondeleteproject,
    ondeletedocument,
    onnewcloudproject,
    onnewclouddocument,
    onnewcloudfolder,
    onrenamecloudfolder,
    ondeletecloudfolder,
    onsignin,
  }: Props = $props();

  const FILTERS_KEY = "typst-desktop-cloud-filters";
  const VIEW_MODE_KEY = "typst-desktop-cloud-view-mode";

  let searchQuery = $state("");
  let filters = $state<CloudFilters>(readStoredPreference(FILTERS_KEY, defaultCloudFilters));
  let viewMode = $state<ViewMode>(readStoredPreference(VIEW_MODE_KEY, "grid"));
  let menu = $state<{ entry: CloudEntry; x: number; y: number } | null>(null);
  let dragging = $state<CloudEntry | null>(null);
  let dropTarget = $state<string | null>(null);
  let thumbnails = $state<Record<string, { kind: string; data: string }>>({});

  $effect(() => writeStoredPreference(FILTERS_KEY, filters));
  $effect(() => writeStoredPreference(VIEW_MODE_KEY, viewMode));

  $effect(() => {
    if (app.account) untrack(() => refreshCloud());
  });

  const handlers: CloudHandlers = {
    openFolder: (folder) => openCloudFolder(folder.id),
    openLocal: (path) => openTarget(path),
    downloadDocument: (documentId, title) => ondownloaddocument(documentId, title),
    removeDownload: (path) => onremovedownload(path),
    deleteDocument: (documentId) => ondeletedocument(documentId),
    cloneProject: (projectId, name) => oncloneproject(projectId, name),
    deleteProject: (projectId) => ondeleteproject(projectId),
    renameFolder: (folder) => onrenamecloudfolder(folder),
    deleteFolder: (folder) => ondeletecloudfolder(folder),
    setFolderColor: (folder, color) => changeFolderColor(folder.id, color),
    downloadFile: (fileId, name) => ondownloadfile(fileId, name),
    openFile: (fileId, name) => onopenfile(fileId, name),
    renameFile: (file) => onrenamefile(file),
    deleteFile: (fileId) => ondeletefile(fileId),
  };

  const inShared = $derived(app.cloudFolder === "shared");
  const folderPath = $derived(inShared ? [] : cloudBreadcrumbs());
  const team = $derived(activeCloudSpace());
  const homeLabel = $derived(team?.name ?? "Home");
  const title = $derived(inShared ? "Shared with me" : (folderPath.at(-1)?.name ?? homeLabel));

  const allEntries = $derived([
    ...app.cloudFolders.map((folder) => folderEntry(folder, handlers)),
    ...app.cloudProjects.map((project) => projectEntry(project, linkedProject(project.id), handlers)),
    ...app.cloudDocuments.map((document) => documentEntry(document, linkedDocument(document.id), handlers)),
    ...app.cloudFiles.map((file) => fileEntry(file, handlers)),
  ]);

  const visibleEntries = $derived(
    sortEntries(allEntries.filter((entry) => matchesSearch(entry, searchQuery)), filters),
  );
  const containerEntries = $derived([
    ...visibleEntries.filter((entry) => entry.kind === "folder"),
    ...visibleEntries.filter((entry) => entry.kind === "project"),
  ]);
  const tileEntries = $derived(
    visibleEntries.filter((entry) => entry.kind === "document" || entry.kind === "file"),
  );

  $effect(() => {
    const pendingPaths = [
      ...app.linkedDocuments.map((linked) => linked.path),
      ...app.linkedProjects.map((linked) => linked.path),
    ];
    let cancelled = false;

    (async () => {
      for (const path of pendingPaths) {
        if (cancelled) return;
        if (thumbnails[path]) continue;
        try {
          const result = await api.thumbnail(path);
          if (!cancelled) thumbnails[path] = result;
        } catch {
          continue;
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  });

  function thumbnailFor(entry: CloudEntry) {
    const local = entry.localPath ? thumbnails[entry.localPath] : undefined;
    if (local) return local;
    return entry.thumbnailSvg ? { kind: "svg", data: entry.thumbnailSvg } : null;
  }

  async function changeFolderColor(folderId: string, color: string | null) {
    try {
      const savedColor = await api.cloudSetFolderColor(folderId, color);
      const withColor = (folder: CloudFolder) =>
        folder.id === folderId ? { ...folder, color: savedColor } : folder;
      app.cloudFolders = app.cloudFolders.map(withColor);
      app.cloudFolderTree = app.cloudFolderTree.map(withColor);
    } catch (error) {
      setError(error);
    }
  }

  function showMenu(entry: CloudEntry, x: number, y: number) {
    menu = { entry, x, y };
  }

  function navigate(folderId: string | null) {
    openCloudFolder(folderId ?? (inShared ? "shared" : null));
  }

  async function moveEntry(entry: CloudEntry, folderId: string | null) {
    try {
      if (entry.kind === "folder") await api.cloudMoveFolder(entry.id, folderId);
      else if (entry.kind === "project") await api.cloudMoveProject(entry.id, folderId);
      else if (entry.kind === "document") await api.cloudMoveDocument(entry.id, folderId);
      else await api.cloudMoveFile(entry.id, folderId);
      await refreshCloud();
    } catch (error) {
      setError(error);
    }
  }

  function startDrag(event: DragEvent, entry: CloudEntry) {
    dragging = entry;
    event.dataTransfer?.setData("text/plain", entry.key);
  }

  function endDrag() {
    dragging = null;
    dropTarget = null;
  }

  function allowDrop(event: DragEvent, folderId: string) {
    if (!dragging || inShared) return;
    if (dragging.kind === "folder" && dragging.id === folderId) return;
    event.preventDefault();
    dropTarget = folderId;
  }

  function leaveDrop(folderId: string) {
    if (dropTarget === folderId) dropTarget = null;
  }

  function dropOn(event: DragEvent, folderId: string) {
    event.preventDefault();
    const entry = dragging;
    endDrag();
    if (entry) moveEntry(entry, folderId === "" ? null : folderId);
  }
</script>

<div class="scroll-thin h-full overflow-y-auto bg-[var(--color-surface-muted)] px-6 pb-6 pt-5">
  {#if !app.account}
    <CloudEmptyState
      icon="ph:cloud-slash"
      heading="Not connected"
      message="Connect a TypstDrive account to sync your documents and projects across devices."
    >
      <button
        class="btn btn-primary"
        onclick={onsignin}
      >
        Sign in
      </button>
    </CloudEmptyState>
  {:else}
    <CloudHeader
      {title}
      {folderPath}
      rootLabel={inShared ? "Shared with me" : homeLabel}
      rootIcon={inShared ? "ph:users-three" : team ? "ph:users-four" : "ph:cloud"}
      showActions={!inShared && canChangeActiveSpace()}
      loading={app.cloudLoading}
      {dropTarget}
      bind:searchQuery
      onnavigate={navigate}
      ondragovercrumb={allowDrop}
      ondragleavecrumb={leaveDrop}
      ondropcrumb={dropOn}
      onnewdocument={onnewclouddocument}
      onnewproject={onnewcloudproject}
      onnewfolder={onnewcloudfolder}
      onupload={onuploadfile}
    />

    {#if app.cloudOffline}
      <div class="mb-4 flex items-center gap-2 rounded-lg border border-[var(--color-warning)]/30 bg-[var(--color-warning)]/10 px-3 py-2 text-xs text-[var(--color-ink)]">
        <Icon icon="ph:wifi-slash" class="text-sm text-[var(--color-warning)]" />
        Offline. Showing the last synced data.
      </div>
    {/if}

    <FilterBar bind:filters bind:viewMode sortOptions={cloudSortOptions} />

    {#if allEntries.length === 0 && app.cloudLoading}
      <div class="flex min-h-[45vh] items-center justify-center text-[var(--color-ink-muted)]">
        <Icon icon="ph:circle-notch" class="animate-spin text-2xl" />
      </div>
    {:else if allEntries.length === 0 && inShared}
      <CloudEmptyState
        icon="ph:users-three"
        heading="Nothing shared yet"
        message="Documents and projects other people share with you will appear here."
      />
    {:else if allEntries.length === 0}
      <CloudEmptyState
        icon={app.cloudFolder === null ? "ph:file-plus" : "ph:folder-open"}
        heading={app.cloudFolder === null ? "Start your first document" : "This folder is empty"}
        message="Create a document, project or folder, or upload images and fonts to use in your Typst files."
      >
        <button
          class="btn btn-primary"
          onclick={onnewclouddocument}
        >
          New document
        </button>
        <button
          class="btn btn-secondary"
          onclick={onnewcloudproject}
        >
          New project
        </button>
      </CloudEmptyState>
    {:else if visibleEntries.length === 0}
      <CloudEmptyState icon="ph:magnifying-glass-minus" heading="No matches" message="Nothing here matches your search." />
    {:else if viewMode === "list"}
      <CloudListView
        entries={[...containerEntries, ...tileEntries]}
        {dropTarget}
        canDrop={!inShared}
        onmenu={showMenu}
        ondragstart={startDrag}
        ondragend={endDrag}
        ondragover={allowDrop}
        ondragleave={leaveDrop}
        ondrop={dropOn}
      />
    {:else}
      {#if containerEntries.length > 0}
        <section class="mb-7">
          <CloudSectionTitle title="Folders & projects" />
          <div class="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-2.5">
            {#each containerEntries as entry (entry.key)}
              <CloudRowCard
                {entry}
                dropActive={dropTarget === entry.id && entry.kind === "folder"}
                canDrop={!inShared}
                onmenu={showMenu}
                ondragstart={startDrag}
                ondragend={endDrag}
                ondragover={allowDrop}
                ondragleave={leaveDrop}
                ondrop={dropOn}
              />
            {/each}
          </div>
        </section>
      {/if}

      {#if tileEntries.length > 0}
        <section>
          <CloudSectionTitle title="Files" />
          <div class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-3">
            {#each tileEntries as entry (entry.key)}
              <CloudTileCard
                {entry}
                thumbnail={thumbnailFor(entry)}
                onmenu={showMenu}
                ondragstart={startDrag}
                ondragend={endDrag}
              />
            {/each}
          </div>
        </section>
      {/if}
    {/if}
  {/if}
</div>

{#if menu}
  <CloudActionMenu
    actions={menu.entry.actions}
    x={menu.x}
    y={menu.y}
    color={menu.entry.color}
    onsetcolor={menu.entry.setColor}
    onclose={() => (menu = null)}
  />
{/if}
