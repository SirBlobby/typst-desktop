<script lang="ts">
  import Icon from "@iconify/svelte";
  import { onMount } from "svelte";
  import { save } from "@tauri-apps/plugin-dialog";
  import { writeImage, writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { listen } from "@tauri-apps/api/event";

  import FileViewer from "$lib/components/FileViewer.svelte";
  import FileTree from "$lib/components/FileTree.svelte";
  import Editor from "$lib/components/Editor.svelte";
  import Preview from "$lib/components/Preview.svelte";
  import PromptModal from "$lib/components/PromptModal.svelte";
  import ConfirmModal from "$lib/components/ConfirmModal.svelte";
  import ConflictModal from "$lib/components/ConflictModal.svelte";
  import LoginModal from "$lib/components/LoginModal.svelte";
  import SettingsModal from "$lib/components/SettingsModal.svelte";
  import AssetsModal from "$lib/components/AssetsModal.svelte";
  import WindowControls from "$lib/components/WindowControls.svelte";
  import ImageViewer from "$lib/components/ImageViewer.svelte";
  import EditorToolbar from "$lib/components/EditorToolbar.svelte";
  import PageSettingsModal from "$lib/components/PageSettingsModal.svelte";

  import type { EditorView } from "@codemirror/view";
  import { insertText } from "$lib/ts/editor-actions";

  import * as api from "$lib/ts/api";
  import type { BrowseEntry, CloudFile, CloudFolder } from "$lib/ts/api";
  import { pickFiles } from "$lib/ts/import";
  import {
    app,
    browseTo,
    clearMessages,
    closeTarget,
    compile,
    downloadCloudFile,
    downloadDocument,
    openFile,
    openTarget,
    refreshAccount,
    refreshCloud,
    refreshEntries,
    refreshTarget,
    removeDownloadedDocument,
    runSync,
    saveAndCompile,
    scheduleAutosave,
    scheduleCompile,
    setError,
    setStatus,
    trackDownload,
  } from "$lib/ts/state.svelte";
  import type { DownloadProgress } from "$lib/ts/state.svelte";

  type Dialog =
    | { kind: "none" }
    | { kind: "new-project" }
    | { kind: "new-folder" }
    | { kind: "new-document" }
    | { kind: "rename-entry"; entry: BrowseEntry }
    | { kind: "delete-entry"; entry: BrowseEntry }
    | { kind: "link-entry"; entry: BrowseEntry }
    | { kind: "save-document-to-cloud"; entry: BrowseEntry }
    | { kind: "new-cloud-project" }
    | { kind: "new-cloud-document" }
    | { kind: "new-cloud-folder" }
    | { kind: "rename-cloud-folder"; folder: CloudFolder }
    | { kind: "delete-cloud-folder"; folder: CloudFolder }
    | { kind: "delete-cloud-project"; id: string }
    | { kind: "delete-cloud-document"; id: string }
    | { kind: "delete-cloud-file"; id: string }
    | { kind: "rename-cloud-file"; file: CloudFile }
    | { kind: "clone-cloud-project"; id: string; name: string }
    | { kind: "new-file"; parent: string }
    | { kind: "new-subfolder"; parent: string }
    | { kind: "rename-file"; path: string }
    | { kind: "delete-file"; paths: string[] }
    | { kind: "login" }
    | { kind: "settings" }
    | { kind: "assets" }
    | { kind: "page-settings" }
    | { kind: "conflicts" };

  let dialog = $state<Dialog>({ kind: "none" });
  let editorView = $state<EditorView | null>(null);
  let imageViewer = $state<{ paths: string[]; index: number } | null>(null);
  let selectedEntries = $state<Set<string>>(new Set());
  let selectedEntry = $state<string | null>(null);
  let selectedIsDir = $state(false);
  let treeDropTarget = $state<string | null>(null);

  const selectedFolder = $derived(
    !selectedEntry
      ? ""
      : selectedIsDir
        ? selectedEntry
        : selectedEntry.includes("/")
          ? selectedEntry.slice(0, selectedEntry.lastIndexOf("/"))
          : "",
  );

  function joinInTarget(parent: string, name: string): string {
    return parent ? `${parent}/${name}` : name;
  }

  function targetChild(relative: string): string {
    return `${app.target?.path}/${relative}`;
  }

  const activeFile = $derived(
    app.target?.files.find((file) => file.path === app.activePath) ?? null,
  );

  function close() {
    dialog = { kind: "none" };
  }

  async function guard(action: () => Promise<void>) {
    try {
      await action();
      close();
    } catch (error) {
      setError(error);
    }
  }

  const createProject = (name: string) =>
    guard(async () => {
      const path = await api.createProjectEntry(app.currentDir, name);
      await refreshEntries();
      await openTarget(path);
    });

  const createFolder = (name: string) =>
    guard(async () => {
      await api.createFolderEntry(app.currentDir, name);
      await refreshEntries();
    });

  const createDocument = (name: string) =>
    guard(async () => {
      const path = await api.createDocumentEntry(app.currentDir, name);
      await refreshEntries();
      await openTarget(path);
    });

  const renameEntry = (entry: BrowseEntry, name: string) =>
    guard(async () => {
      await api.renameEntry(entry.path, name);
      await refreshEntries();
    });

  const deleteEntry = (entry: BrowseEntry) =>
    guard(async () => {
      await api.deleteEntry(entry.path);
      await refreshEntries();
      setStatus(`Deleted '${entry.name}'`);
    });

  const currentCloudFolder = () =>
    app.cloudFolder === "shared" ? null : app.cloudFolder;

  const linkEntry = (entry: BrowseEntry) =>
    guard(async () => {
      const report = await api.cloudLinkProject(entry.path);
      await refreshEntries();
      await refreshCloud();
      setStatus(`Uploaded ${report.pushed.length} files to a new cloud project`);
    });

  const saveDocumentToCloud = (entry: BrowseEntry, title: string) =>
    guard(async () => {
      await api.cloudCreateDocument(entry.path, title);
      await refreshEntries();
      setStatus(`Saved '${entry.name}' to the cloud`);
    });

  const createCloudProject = (name: string) =>
    guard(async () => {
      await api.cloudCreateProject(name, currentCloudFolder());
      await refreshCloud();
    });

  const createCloudDocument = (title: string) =>
    guard(async () => {
      await api.cloudNewDocument(title, currentCloudFolder());
      await refreshCloud();
    });

  const createCloudFolder = (name: string) =>
    guard(async () => {
      await api.cloudCreateFolder(name, currentCloudFolder());
      await refreshCloud();
    });

  const renameCloudFolder = (folder: CloudFolder, name: string) =>
    guard(async () => {
      await api.cloudRenameFolder(folder.id, name);
      await refreshCloud();
    });

  const deleteCloudFolder = (folder: CloudFolder) =>
    guard(async () => {
      await api.cloudDeleteFolder(folder.id);
      await refreshCloud();
    });

  const deleteCloudProject = (id: string) =>
    guard(async () => {
      await api.cloudDeleteProject(id);
      await refreshCloud();
    });

  const deleteCloudDocument = (id: string) =>
    guard(async () => {
      await api.cloudDeleteDocument(id);
      await refreshCloud();
    });

  const deleteCloudFile = (id: string) =>
    guard(async () => {
      await api.cloudDeleteFile(id);
      await refreshCloud();
    });

  const uploadCloudFiles = () =>
    guard(async () => {
      const sources = await pickFiles("assets");
      if (sources.length === 0) return;

      const folderId = currentCloudFolder();
      for (const source of sources) {
        await api.cloudUploadFile(source, folderId);
      }
      await refreshCloud();
      setStatus(`Uploaded ${sources.length} file(s) to TypstDrive`);
    });

  const renameCloudFile = (file: CloudFile, name: string) =>
    guard(async () => {
      await api.cloudRenameFile(file.id, name);
      await refreshCloud();
    });

  const cloneCloudProject = (id: string, name: string) =>
    guard(async () => {
      const parent = app.currentDir;
      await api.cloudCloneProject(id, name, parent);
      app.scope = "local";
      await browseTo(parent);
      setStatus(`Downloaded '${name}' to this device`);
    });

  async function importFiles(folder?: string) {
    const sources = await pickFiles("all");
    if (sources.length === 0) return;

    try {
      if (app.view === "editor" && app.target) {
        const destination = folder ?? selectedFolder;
        const imported = await api.importIntoFolder(
          destination ? targetChild(destination) : app.target.path,
          sources,
        );
        await refreshTarget();
        await compile();
        setStatus(`Imported ${imported.length} file(s)`);
      } else {
        const imported = await api.importIntoFolder(app.currentDir, sources);
        await refreshEntries();
        setStatus(`Imported ${imported.length} file(s)`);
      }
    } catch (error) {
      setError(error);
    }
  }

  const createFileInTarget = (parent: string, name: string) =>
    guard(async () => {
      const file = name.includes(".") ? name : `${name}.typ`;
      const path = joinInTarget(parent, file);
      await api.writeTargetFile(app.target!.path, path, "");
      await refreshTarget();
      await openFile(path);
    });

  const createFolderInTarget = (parent: string, name: string) =>
    guard(async () => {
      await api.createFolderEntry(app.target!.path, joinInTarget(parent, name));
      await refreshTarget();
    });

  const moveInTarget = (path: string, destination: string) =>
    guard(async () => {
      await api.moveEntry(targetChild(path), targetChild(destination));
      await refreshTarget();
      if (app.activePath === path) {
        const name = path.split("/").pop() ?? path;
        await openFile(joinInTarget(destination, name));
      }
      await compile();
    });

  const duplicateInTarget = (path: string) =>
    guard(async () => {
      await api.duplicateEntry(targetChild(path));
      await refreshTarget();
    });

  async function revealInTarget(path: string) {
    try {
      const absolute = await api.absolutePath(targetChild(path));
      await revealItemInDir(absolute);
    } catch (error) {
      setError(error);
    }
  }

  const renameFile = (path: string, next: string) =>
    guard(async () => {
      const payload = await api.readTargetFile(app.target!.path, path);
      await api.writeTargetFile(app.target!.path, next, payload.content);
      await api.deleteEntry(`${app.target!.path}/${path}`);
      await refreshTarget();
      if (app.activePath === path) await openFile(next);
    });

  const deleteFile = (paths: string[]) =>
    guard(async () => {
      for (const path of paths) {
        await api.deleteEntry(`${app.target!.path}/${path}`);
        if (app.activePath === path) {
          app.activePath = null;
          app.editorContent = "";
        }
      }
      selectedEntries = new Set();
      selectedEntry = null;
      await refreshTarget();
      await compile();
    });

  async function setEntrypoint(path: string) {
    if (!app.target) return;
    try {
      await api.setTargetEntrypoint(app.target.path, path);
      await refreshTarget();
      await compile();
      setStatus(`'${path}' is now the entrypoint`);
    } catch (error) {
      setError(error);
    }
  }

  async function exportAs(format: string) {
    if (!app.target) return;

    const name = app.target.path.split("/").pop() ?? "document";
    const destination = await save({
      defaultPath: `${name.replace(/\.typ$/i, "")}.${format}`,
      filters: [{ name: format.toUpperCase(), extensions: [format] }],
    });
    if (!destination) return;

    try {
      await api.exportTarget(app.target.path, format, destination);
      setStatus(`Exported to ${destination}`);
    } catch (error) {
      setError(error);
    }
  }

  async function copyAs(format: string) {
    if (!app.target) return;

    try {
      if (format === "png") {
        const bytes = await api.renderTargetPng(app.target.path);
        await writeImage(new Uint8Array(bytes));
      } else if (format === "svg") {
        const svg = app.compiled?.pages[0];
        if (!svg) {
          setStatus("Nothing to copy yet");
          return;
        }
        await writeText(svg);
      }
      setStatus(`Copied ${format.toUpperCase()} to clipboard`);
    } catch (error) {
      setError(error);
    }
  }

  const resolveDocumentConflicts = (resolutions: api.Resolution[]) =>
    guard(async () => {
      for (const resolution of resolutions) {
        await api.cloudResolveDocument(
          app.target!.path,
          resolution.content,
          resolution.server_hash,
        );
      }
      app.conflicts = [];
      if (app.activePath) await openFile(app.activePath);
      setStatus("Conflicts resolved and uploaded");
    });

  const resolveConflicts = (resolutions: api.Resolution[]) =>
    guard(async () => {
      const report = await api.cloudResolveConflicts(
        app.target!.path,
        resolutions,
      );
      app.conflicts = report.conflicts;
      await refreshTarget();
      if (app.activePath) await openFile(app.activePath);
      setStatus("Conflicts resolved and uploaded");
    });

  function handleKeydown(event: KeyboardEvent) {
    if ((event.metaKey || event.ctrlKey) && event.key === "s") {
      event.preventDefault();
      if (app.view === "editor") saveAndCompile();
    }
  }

  let dropActive = $state(false);

  function dropDestination():
    | { kind: "folder"; path: string }
    | { kind: "target"; path: string }
    | null {
    if (app.view === "editor" && app.target) {
      return { kind: "target", path: app.target.path };
    }
    if (app.view === "files" && app.scope === "local") {
      return { kind: "folder", path: app.currentDir };
    }
    return null;
  }

  function folderUnderPointer(x: number, y: number): string | null {
    const element = document
      .elementFromPoint(x, y)
      ?.closest("[data-tree-path]") as HTMLElement | null;
    if (!element) return null;
    if (element.dataset.treeDir !== "true") return null;
    return element.dataset.treePath ?? "";
  }

  async function dropPaths(paths: string[], folder: string | null) {
    const destination = dropDestination();
    if (!destination || paths.length === 0) return;

    try {
      const imported =
        destination.kind === "target"
          ? await api.importIntoFolder(
              folder ? targetChild(folder) : destination.path,
              paths,
            )
          : await api.importIntoFolder(destination.path, paths);

      if (destination.kind === "target") {
        await refreshTarget();
        await compile();
      } else {
        await refreshEntries();
      }
      setStatus(`Imported ${imported.length} item(s)`);
    } catch (error) {
      setError(error);
    }
  }

  onMount(() => {
    const downloads = listen<DownloadProgress>("download://progress", (event) =>
      trackDownload(event.payload),
    );

    const pending = getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "over") {
        dropActive = dropDestination() !== null;
        treeDropTarget =
          app.view === "editor"
            ? folderUnderPointer(
                event.payload.position.x,
                event.payload.position.y,
              )
            : null;
      } else if (event.payload.type === "drop") {
        const folder = treeDropTarget;
        dropActive = false;
        treeDropTarget = null;
        dropPaths(event.payload.paths, folder);
      } else {
        dropActive = false;
        treeDropTarget = null;
      }
    });

    return () => {
      pending.then((unlisten) => unlisten());
      downloads.then((unlisten) => unlisten());
    };
  });

  const lspLabel: Record<string, string> = {
    off: "LSP off",
    starting: "LSP starting",
    on: "LSP ready",
    unavailable: "LSP unavailable",
  };
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="flex h-screen flex-col">
  <header
    data-tauri-drag-region
    class="flex h-11 shrink-0 select-none items-center gap-2 border-b border-[var(--color-line)] bg-[var(--color-surface)] pl-3"
  >
    {#if app.view === "editor"}
      <button
        class="flex items-center gap-1.5 rounded-md px-2 py-1.5 text-xs text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)]"
        onclick={closeTarget}
      >
        <Icon icon="ph:arrow-left" />
        Files
      </button>
      <span data-tauri-drag-region class="text-sm font-medium">
        {app.target?.path.split("/").pop()}
      </span>
      {#if app.target?.standalone}
        <span
          class="rounded bg-[var(--color-surface-sunken)] px-1.5 py-0.5 text-[10px] text-[var(--color-ink-muted)]"
        >
          single file
        </span>
      {/if}
      {#if app.dirty}
        <span class="h-1.5 w-1.5 rounded-full bg-[var(--color-accent)]"></span>
      {/if}
    {:else}
      <span
        class="flex h-6 w-6 items-center justify-center rounded-md bg-[var(--color-accent)]"
      >
        <img src="/favicon.png" alt="Typst Desktop" class="h-6 w-6" />
      </span>
      <span data-tauri-drag-region class="text-sm font-semibold">
        Typst Desktop
      </span>
    {/if}

    <div data-tauri-drag-region class="h-full flex-1"></div>

    {#if app.view !== "editor"}
      <div class="flex rounded-lg bg-[var(--color-surface-sunken)] p-0.5 text-xs font-medium">
        {#each [["local", "Local", "ph:hard-drives"], ["cloud", "Cloud", "ph:cloud"]] as [value, label, icon]}
          <button
            class="flex items-center gap-1.5 rounded-md px-3 py-1.5 transition
              {app.scope === value
              ? 'bg-[var(--color-surface)] text-[var(--color-ink)] shadow-sm'
              : 'text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
            onclick={() => (app.scope = value as "local" | "cloud")}
          >
            <Icon {icon} />
            {label}
          </button>
        {/each}
      </div>
    {/if}

    {#if app.view === "editor"}
      <span
        class="flex items-center gap-1 text-[10px] text-[var(--color-ink-muted)]"
        title="Typst language server (tinymist)"
      >
        <span
          class="h-1.5 w-1.5 rounded-full
            {app.lspStatus === 'on'
            ? 'bg-[var(--color-success)]'
            : app.lspStatus === 'starting'
              ? 'bg-[var(--color-accent)]'
              : 'bg-[var(--color-ink-muted)]'}"
        ></span>
        {lspLabel[app.lspStatus]}
      </span>

      <button
        class="flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)]"
        onclick={saveAndCompile}
      >
        <Icon icon="ph:floppy-disk" />
        Save
      </button>

      <div class="group relative">
        <button
          class="flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)]"
        >
          <Icon icon="ph:export" />
          Export
        </button>
        <div
          class="invisible absolute right-0 top-full z-20 flex w-32 flex-col rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] py-1 text-xs shadow-lg group-hover:visible"
        >
          {#each ["pdf", "png", "html"] as format}
            <button
              class="px-3 py-1.5 text-left uppercase hover:bg-[var(--color-surface-sunken)]"
              onclick={() => exportAs(format)}
            >
              {format}
            </button>
          {/each}
        </div>
      </div>

      <div class="group relative">
        <button
          class="flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)]"
        >
          <Icon icon="ph:copy" />
          Copy
        </button>
        <div
          class="invisible absolute right-0 top-full z-20 flex w-32 flex-col rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] py-1 text-xs shadow-lg group-hover:visible"
        >
          {#each ["png", "svg"] as format}
            <button
              class="px-3 py-1.5 text-left uppercase hover:bg-[var(--color-surface-sunken)]"
              onclick={() => copyAs(format)}
            >
              {format}
            </button>
          {/each}
        </div>
      </div>

      {#if app.target?.cloud_project_id || app.documentLink}
        <button
          class="flex items-center gap-1.5 rounded-md bg-[var(--color-accent)] px-2.5 py-1.5 text-xs font-medium text-white transition hover:opacity-90 disabled:opacity-50"
          disabled={app.syncing}
          onclick={() => runSync("sync")}
        >
          <Icon
            icon={app.syncing ? "ph:circle-notch" : "ph:arrows-clockwise"}
            class={app.syncing ? "animate-spin" : ""}
          />
          Sync
        </button>
      {/if}
    {/if}

    <button
      class="rounded-md p-1.5 text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)]"
      onclick={() => (dialog = { kind: "settings" })}
      aria-label="Settings"
    >
      <Icon icon="ph:gear-six" />
    </button>

    <div class="ml-1 h-5 w-px bg-[var(--color-line)]"></div>

    <WindowControls />
  </header>

  {#if app.download}
    {@const progress = app.download}
    <div
      class="flex shrink-0 items-center gap-3 border-b border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2"
    >
      <Icon
        icon={progress.done ? "ph:check-circle" : "ph:cloud-arrow-down"}
        class="shrink-0 text-base {progress.done
          ? 'text-[var(--color-success)]'
          : 'text-[var(--color-accent)]'}"
      />

      <span class="shrink-0 text-xs">
        {progress.done ? "Downloaded" : "Downloading"}
        <span class="font-medium">{progress.label}</span>
      </span>

      <div
        class="h-1.5 min-w-0 flex-1 overflow-hidden rounded-full bg-[var(--color-surface-sunken)]"
      >
        <div
          class="h-full rounded-full transition-all duration-200
            {progress.done
            ? 'bg-[var(--color-success)]'
            : 'bg-[var(--color-accent)]'}"
          style="width: {progress.total > 0
            ? Math.round((progress.current / progress.total) * 100)
            : 0}%"
        ></div>
      </div>

      {#if progress.total > 1}
        <span
          class="shrink-0 tabular-nums text-[10px] text-[var(--color-ink-muted)]"
        >
          {progress.current} of {progress.total} files
        </span>
      {/if}
    </div>
  {/if}

  {#if app.status || app.error}
    <div
      class="flex shrink-0 items-center gap-2 border-b px-3 py-1.5 text-xs
        {app.error
        ? 'border-[var(--color-danger)]/30 bg-[var(--color-danger)]/10 text-[var(--color-danger)]'
        : 'border-[var(--color-success)]/30 bg-[var(--color-success)]/10 text-[var(--color-success)]'}"
    >
      <Icon icon={app.error ? "ph:warning-circle" : "ph:check-circle"} />
      <span class="flex-1">{app.error || app.status}</span>
      {#if app.conflicts.length > 0}
        <button
          class="rounded border border-current px-2 py-0.5 font-medium"
          onclick={() => (dialog = { kind: "conflicts" })}
        >
          Resolve
        </button>
      {/if}
      <button onclick={clearMessages} aria-label="Dismiss">
        <Icon icon="ph:x" />
      </button>
    </div>
  {/if}

  <div class="flex min-h-0 flex-1">
    {#if app.view === "files"}
      <div class="flex-1">
        <FileViewer
          onnewproject={() => (dialog = { kind: "new-project" })}
          onnewfolder={() => (dialog = { kind: "new-folder" })}
          onnewdocument={() => (dialog = { kind: "new-document" })}
          onupload={importFiles}
          onrename={(entry) => (dialog = { kind: "rename-entry", entry })}
          ondelete={(entry) => (dialog = { kind: "delete-entry", entry })}
          onlink={(entry) => (dialog = { kind: "link-entry", entry })}
          onsavetocloud={(entry) =>
            (dialog = { kind: "save-document-to-cloud", entry })}
          onviewimage={(paths, index) => (imageViewer = { paths, index })}
          ondownloaddocument={(documentId, title) =>
            downloadDocument(documentId, title)}
          onremovedownload={removeDownloadedDocument}
          ondownloadfile={downloadCloudFile}
          ondeletefile={(id) => (dialog = { kind: "delete-cloud-file", id })}
          onuploadfile={uploadCloudFiles}
          onrenamefile={(file) => (dialog = { kind: "rename-cloud-file", file })}
          onnewcloudproject={() => (dialog = { kind: "new-cloud-project" })}
          onnewclouddocument={() => (dialog = { kind: "new-cloud-document" })}
          onnewcloudfolder={() => (dialog = { kind: "new-cloud-folder" })}
          onrenamecloudfolder={(folder) =>
            (dialog = { kind: "rename-cloud-folder", folder })}
          ondeletecloudfolder={(folder) =>
            (dialog = { kind: "delete-cloud-folder", folder })}
          oncloneproject={(id, name) =>
            (dialog = { kind: "clone-cloud-project", id, name })}
          ondeleteproject={(id) => (dialog = { kind: "delete-cloud-project", id })}
          ondeletedocument={(id) => (dialog = { kind: "delete-cloud-document", id })}
          onsignin={() => (dialog = { kind: "login" })}
        />
      </div>
    {:else}
      {#if !app.target?.standalone}
        <div
          class="flex w-56 shrink-0 flex-col border-r border-[var(--color-line)] bg-[var(--color-surface)]"
        >
          <div
            class="flex items-center justify-between border-b border-[var(--color-line)] px-3 py-1.5"
          >
            <span
              class="truncate text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]"
            >
              {selectedFolder ? selectedFolder : "Files"}
            </span>
          </div>

          <FileTree
            files={app.target?.files ?? []}
            activePath={app.activePath}
            entrypoint={app.target?.entrypoint ?? "main.typ"}
            selected={selectedEntries}
            dropTarget={treeDropTarget}
            onopen={openFile}
            onselect={(paths, primary, isDir) => {
              selectedEntries = new Set(paths);
              selectedEntry = primary;
              selectedIsDir = isDir;
            }}
            onrename={(path) => (dialog = { kind: "rename-file", path })}
            ondelete={(paths) => (dialog = { kind: "delete-file", paths })}
            onduplicate={duplicateInTarget}
            onreveal={revealInTarget}
            onsetentry={setEntrypoint}
            onmove={moveInTarget}
            onnewfile={(parent) => (dialog = { kind: "new-file", parent })}
            onnewfolder={(parent) => (dialog = { kind: "new-subfolder", parent })}
            onimport={(parent) => importFiles(parent)}
          />
        </div>
      {/if}

      <div class="flex min-w-0 flex-1">
        <div class="flex min-w-0 flex-1 flex-col border-r border-[var(--color-line)]">
          {#if app.activePath && activeFile?.is_text}
            <EditorToolbar
              view={editorView}
              disabled={app.activePath.toLowerCase().endsWith(".toml")}
              onassets={() => (dialog = { kind: "assets" })}
              onpagesettings={() => (dialog = { kind: "page-settings" })}
            />

            <div
              class="flex shrink-0 items-center gap-2 border-b border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-xs"
            >
              <Icon icon="ph:file-text" class="text-[var(--color-ink-muted)]" />
              <span>{app.activePath}</span>
              {#if app.dirty}
                <span class="text-[var(--color-ink-muted)]">edited</span>
              {/if}
            </div>

            {#key app.target?.path + ":" + app.activePath}
              <div class="min-h-0 flex-1">
                <Editor
                  content={app.editorContent}
                  filePath={app.activePath}
                  targetPath={app.target?.path ?? ""}
                  diagnostics={app.diagnostics}
                  onchange={(value) => {
                    app.editorContent = value;
                    app.dirty = true;
                    scheduleCompile();
                    scheduleAutosave();
                  }}
                  onsave={saveAndCompile}
                  onlspstatus={(status) => (app.lspStatus = status)}
                  onready={(view) => (editorView = view)}
                />
              </div>
            {/key}
          {:else}
            <div
              class="flex flex-1 flex-col items-center justify-center gap-2 bg-[var(--color-surface)] text-[var(--color-ink-muted)]"
            >
              <Icon icon="ph:file-dashed" class="text-4xl" />
              <p class="text-sm">
                {app.activePath
                  ? "This file cannot be edited as text"
                  : "Select a file to edit"}
              </p>
            </div>
          {/if}
        </div>

        <div class="min-w-0 flex-1">
          <Preview
            compiled={app.compiled}
            diagnostics={app.diagnostics}
            compiling={app.compiling}
          />
        </div>
      </div>
    {/if}
  </div>
</div>

{#if imageViewer}
  <ImageViewer
    paths={imageViewer.paths}
    index={imageViewer.index}
    onclose={() => (imageViewer = null)}
  />
{/if}

{#if dropActive}
  <div
    class="pointer-events-none fixed inset-0 z-40 flex items-center justify-center bg-[var(--color-accent)]/10 p-8"
  >
    <div
      class="flex flex-col items-center gap-3 rounded-xl border-2 border-dashed border-[var(--color-accent)] bg-[var(--color-surface)]/95 px-10 py-8 shadow-xl"
    >
      <Icon icon="ph:tray-arrow-down" class="text-4xl text-[var(--color-accent)]" />
      <p class="text-sm font-medium">
        {app.view === "editor"
          ? `Drop to add files to ${app.target?.path.split("/").pop()}`
          : app.currentDir
            ? `Drop to add files to ${app.currentDir.split("/").pop()}`
            : "Drop to add files to your workspace"}
      </p>
      <p class="text-xs text-[var(--color-ink-muted)]">
        Images, fonts, and Typst files are copied in.
      </p>
    </div>
  </div>
{/if}

{#if dialog.kind === "new-project"}
  <PromptModal
    title="New project"
    label="Project name"
    icon="ph:folder-plus"
    placeholder="My thesis"
    onsubmit={createProject}
    onclose={close}
  />
{:else if dialog.kind === "new-folder"}
  <PromptModal
    title="New folder"
    label="Folder name"
    icon="ph:folder-plus"
    onsubmit={createFolder}
    onclose={close}
  />
{:else if dialog.kind === "new-document"}
  <PromptModal
    title="New document"
    label="Document name"
    icon="ph:file-plus"
    placeholder="notes"
    suffix=".typ"
    onsubmit={createDocument}
    onclose={close}
  />
{:else if dialog.kind === "rename-entry"}
  {@const target = dialog}
  <PromptModal
    title="Rename"
    label="New name"
    value={target.entry.name}
    confirmLabel="Rename"
    onsubmit={(name) => renameEntry(target.entry, name)}
    onclose={close}
  />
{:else if dialog.kind === "delete-entry"}
  {@const target = dialog}
  <ConfirmModal
    title="Delete"
    message="'{target.entry.name}' will be permanently removed from this device."
    onconfirm={() => deleteEntry(target.entry)}
    onclose={close}
  />
{:else if dialog.kind === "link-entry"}
  {@const target = dialog}
  <ConfirmModal
    title="Upload to cloud"
    message="A new cloud project will be created for '{target.entry.name}' and its files uploaded."
    confirmLabel="Upload"
    onconfirm={() => linkEntry(target.entry)}
    onclose={close}
  />
{:else if dialog.kind === "save-document-to-cloud"}
  {@const target = dialog}
  <PromptModal
    title="Save to cloud"
    label="Document title"
    icon="ph:cloud-arrow-up"
    value={target.entry.name.replace(/\.typ$/i, "")}
    confirmLabel="Save"
    onsubmit={(title) => saveDocumentToCloud(target.entry, title)}
    onclose={close}
  />
{:else if dialog.kind === "new-cloud-project"}
  <PromptModal
    title="New cloud project"
    label="Project name"
    icon="ph:cloud-plus"
    onsubmit={createCloudProject}
    onclose={close}
  />
{:else if dialog.kind === "new-cloud-document"}
  <PromptModal
    title="New cloud document"
    label="Document title"
    icon="ph:cloud-plus"
    onsubmit={createCloudDocument}
    onclose={close}
  />
{:else if dialog.kind === "new-cloud-folder"}
  <PromptModal
    title="New cloud folder"
    label="Folder name"
    icon="ph:folder-plus"
    onsubmit={createCloudFolder}
    onclose={close}
  />
{:else if dialog.kind === "rename-cloud-folder"}
  {@const target = dialog}
  <PromptModal
    title="Rename folder"
    label="New name"
    value={target.folder.name}
    confirmLabel="Rename"
    onsubmit={(name) => renameCloudFolder(target.folder, name)}
    onclose={close}
  />
{:else if dialog.kind === "delete-cloud-folder"}
  {@const target = dialog}
  <ConfirmModal
    title="Delete cloud folder"
    message="'{target.folder.name}' will be permanently removed from TypstDrive. It must be empty first."
    onconfirm={() => deleteCloudFolder(target.folder)}
    onclose={close}
  />
{:else if dialog.kind === "delete-cloud-project"}
  {@const target = dialog}
  <ConfirmModal
    title="Delete cloud project"
    message="This permanently deletes the project and its files from TypstDrive. Local copies are kept."
    onconfirm={() => deleteCloudProject(target.id)}
    onclose={close}
  />
{:else if dialog.kind === "delete-cloud-document"}
  {@const target = dialog}
  <ConfirmModal
    title="Delete cloud document"
    message="This permanently deletes the document from TypstDrive. A local copy, if downloaded, is kept."
    onconfirm={() => deleteCloudDocument(target.id)}
    onclose={close}
  />
{:else if dialog.kind === "delete-cloud-file"}
  {@const target = dialog}
  <ConfirmModal
    title="Delete cloud file"
    message="This permanently deletes the file from TypstDrive."
    onconfirm={() => deleteCloudFile(target.id)}
    onclose={close}
  />
{:else if dialog.kind === "rename-cloud-file"}
  {@const target = dialog}
  <PromptModal
    title="Rename file"
    label="New name"
    value={target.file.name}
    confirmLabel="Rename"
    onsubmit={(name) => renameCloudFile(target.file, name)}
    onclose={close}
  />
{:else if dialog.kind === "clone-cloud-project"}
  {@const target = dialog}
  <PromptModal
    title="Download project"
    label="Save as project"
    icon="ph:download-simple"
    value={target.name}
    confirmLabel="Download"
    onsubmit={(name) => cloneCloudProject(target.id, name)}
    onclose={close}
  />
{:else if dialog.kind === "new-file"}
  {@const target = dialog}
  <PromptModal
    title={target.parent ? `New file in ${target.parent}` : "New file"}
    label="File name"
    icon="ph:file-plus"
    placeholder="chapter-1"
    onsubmit={(name) => createFileInTarget(target.parent, name)}
    onclose={close}
  />
{:else if dialog.kind === "new-subfolder"}
  {@const target = dialog}
  <PromptModal
    title={target.parent ? `New folder in ${target.parent}` : "New folder"}
    label="Folder name"
    icon="ph:folder-plus"
    placeholder="figures"
    onsubmit={(name) => createFolderInTarget(target.parent, name)}
    onclose={close}
  />
{:else if dialog.kind === "rename-file"}
  {@const target = dialog}
  <PromptModal
    title="Rename file"
    label="New path"
    value={target.path}
    confirmLabel="Rename"
    onsubmit={(next) => renameFile(target.path, next)}
    onclose={close}
  />
{:else if dialog.kind === "delete-file"}
  {@const target = dialog}
  <ConfirmModal
    title={target.paths.length > 1 ? "Delete files" : "Delete file"}
    message={target.paths.length > 1
      ? `${target.paths.length} items will be permanently deleted.`
      : `'${target.paths[0]}' will be permanently deleted.`}
    onconfirm={() => deleteFile(target.paths)}
    onclose={close}
  />
{:else if dialog.kind === "login"}
  <LoginModal
    serverUrl={app.settings?.server_url ?? ""}
    onsuccess={async () => {
      close();
      await refreshAccount();
      app.scope = "cloud";
      setStatus("Connected to TypstDrive");
    }}
    onclose={close}
  />
{:else if dialog.kind === "settings"}
  <SettingsModal onclose={close} onsignin={() => (dialog = { kind: "login" })} />
{:else if dialog.kind === "assets"}
  <AssetsModal
    oninsert={app.view === "editor" && editorView
      ? (snippet) => {
          insertText(editorView, snippet);
          close();
        }
      : undefined}
    onchanged={compile}
    onclose={close}
  />
{:else if dialog.kind === "page-settings"}
  <PageSettingsModal view={editorView} onclose={close} />
{:else if dialog.kind === "conflicts"}
  <ConflictModal
    conflicts={app.conflicts}
    onresolve={app.documentLink ? resolveDocumentConflicts : resolveConflicts}
    onclose={close}
  />
{/if}
