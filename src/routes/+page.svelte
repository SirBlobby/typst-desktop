<script lang="ts">
  import Icon from "@iconify/svelte";
  import { onMount } from "svelte";
  import { registerHotkey, unregisterAll } from "$lib/ts/hotkeys";
  import { save } from "@tauri-apps/plugin-dialog";
  import { writeImage, writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { Image } from "@tauri-apps/api/image";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { listen } from "@tauri-apps/api/event";

  import FileViewer from "$lib/components/FileViewer.svelte";
  import CloudView from "$lib/components/cloud/CloudView.svelte";
  import FileTree from "$lib/components/FileTree.svelte";
  import BrowserSidebar from "$lib/components/BrowserSidebar.svelte";
  import Editor from "$lib/components/Editor.svelte";
  import Preview from "$lib/components/Preview.svelte";
  import PromptModal from "$lib/components/PromptModal.svelte";
  import ConfirmModal from "$lib/components/ConfirmModal.svelte";
  import ConflictModal from "$lib/components/ConflictModal.svelte";
  import LoginModal from "$lib/components/LoginModal.svelte";
  import SettingsModal from "$lib/components/SettingsModal.svelte";
  import AssetsModal from "$lib/components/AssetsModal.svelte";
  import WindowControls from "$lib/components/WindowControls.svelte";
  import FileViewerOverlay from "$lib/components/viewer/FileViewerOverlay.svelte";
  import ImageView from "$lib/components/viewer/ImageView.svelte";
  import PdfView from "$lib/components/viewer/PdfView.svelte";
  import EditorToolbar from "$lib/components/EditorToolbar.svelte";
  import ExportMenu from "$lib/components/ExportMenu.svelte";
  import PreviewModeMenu from "$lib/components/PreviewModeMenu.svelte";
  import LanguagePicker from "$lib/components/LanguagePicker.svelte";
  import { isLatexPath, languageLabel, sourceExtension, type DocumentLanguage } from "$lib/ts/document-language";
  import { previewMode } from "$lib/ts/preview-mode.svelte";
  import PageSettingsModal from "$lib/components/PageSettingsModal.svelte";
  import InsertDialogs from "$lib/components/editor/insert/InsertDialogs.svelte";
  import ReviewLayer from "$lib/components/review/ReviewLayer.svelte";
  import ReviewBar from "$lib/components/review/ReviewBar.svelte";
  import ReviewPanel from "$lib/components/review/ReviewPanel.svelte";
  import { reviewExtensions } from "$lib/ts/review/extensions";

  import type { EditorView } from "@codemirror/view";
  import { insertText, redoEdit, undoEdit } from "$lib/ts/editor-actions";
  import { formattingCommandIds, runFormattingCommand } from "$lib/ts/editor/formatting-commands";

  import * as api from "$lib/ts/api";
  import type { BrowseEntry, CloudFile, CloudFolder } from "$lib/ts/api";
  import { pickFiles } from "$lib/ts/import";
  import { dropDestination, dropMessage, dropPaths } from "$lib/ts/drop";
  import { exportFormat } from "$lib/ts/export-formats";
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
    resolveCollabConflict,
    cancelCollabConflict,
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
  let createLanguage = $state<DocumentLanguage>("typst");
  let editorView = $state<EditorView | null>(null);
  const editorExtensions = reviewExtensions();
  let fileViewer = $state<{ paths: string[]; index: number } | null>(null);
  let selectedEntries = $state<Set<string>>(new Set());
  let selectedEntry = $state<string | null>(null);
  let selectedIsDir = $state(false);
  let treeDropTarget = $state<string | null>(null);

  const SIDEBAR_COLLAPSED_KEY = "editor-sidebar-collapsed";
  let sidebarCollapsed = $state(
    localStorage.getItem(SIDEBAR_COLLAPSED_KEY) === "true",
  );

  function toggleSidebar() {
    sidebarCollapsed = !sidebarCollapsed;
    localStorage.setItem(SIDEBAR_COLLAPSED_KEY, String(sidebarCollapsed));
  }

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
      const path = await api.createProjectEntry(app.currentDir, name, createLanguage);
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
      const path = await api.createDocumentEntry(app.currentDir, name, createLanguage);
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
      await api.cloudCreateProject(name, currentCloudFolder(), createLanguage);
      await refreshCloud();
    });

  const createCloudDocument = (title: string) =>
    guard(async () => {
      await api.cloudNewDocument(title, currentCloudFolder(), createLanguage);
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

  async function openCloudFile(fileId: string, name: string) {
    const path = await downloadCloudFile(fileId, name);
    if (!path) return;
    if (api.isViewablePath(name)) {
      fileViewer = { paths: [path], index: 0 };
    } else {
      setStatus(`'${name}' is in your shared assets. Insert it from the Assets panel in the editor`);
    }
  }

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

  async function convertDocuments() {
    const sources = await pickFiles("convertible");
    if (sources.length === 0) return;

    try {
      setStatus("Converting with pandoc...");
      const created: string[] = [];
      for (const source of sources) {
        created.push(await api.pandocImport(app.currentDir, source));
      }
      await refreshEntries();
      if (created.length === 1) await openTarget(created[0]);
      setStatus(`Converted ${created.length} document(s) to Typst`);
    } catch (error) {
      setError(error);
    }
  }

  async function exportAs(formatId: string) {
    const format = exportFormat(formatId);
    if (!app.target || !format) return;

    const sourceFile = format.id === "typ" ? app.activePath : null;
    const baseName = (sourceFile ?? app.target.path).split("/").pop() ?? "document";
    const extension = sourceFile && isLatexPath(sourceFile) ? "tex" : format.extension;
    const destination = await save({
      defaultPath: `${baseName.replace(/\.(typ|tex)$/i, "")}.${extension}`,
      filters: [{ name: format.label, extensions: [extension] }],
    });
    if (!destination) return;

    try {
      if (app.dirty && !app.collabIntent) await saveAndCompile();
      setStatus(`Exporting ${format.label}...`);
      await api.exportTarget(app.target.path, format.id, destination, sourceFile);
      setStatus(`Exported to ${destination}`);
    } catch (error) {
      setError(error);
    }
  }

  async function pngBytesToRgba(bytes: Uint8Array) {
    const blob = new Blob([new Uint8Array(bytes)], { type: "image/png" });
    const bitmap = await createImageBitmap(blob);
    const canvas = document.createElement("canvas");
    canvas.width = bitmap.width;
    canvas.height = bitmap.height;
    const context = canvas.getContext("2d");
    if (!context) throw new Error("Could not create canvas context");
    context.drawImage(bitmap, 0, 0);
    const { data } = context.getImageData(0, 0, canvas.width, canvas.height);
    return { rgba: new Uint8Array(data.buffer), width: canvas.width, height: canvas.height };
  }

  async function copyAs(format: string) {
    if (!app.target) return;

    try {
      if (format === "png") {
        const bytes = await api.renderTargetPng(app.target.path);
        const { rgba, width, height } = await pngBytesToRgba(new Uint8Array(bytes));
        const image = await Image.new(rgba, width, height);
        await writeImage(image);
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

  let draggedPaths = $state<string[]>([]);
  let dropLabel = $state<string | null>(null);

  function folderUnderPointer(x: number, y: number): string | null {
    const element = document
      .elementFromPoint(x, y)
      ?.closest("[data-tree-path]") as HTMLElement | null;
    if (!element) return null;
    if (element.dataset.treeDir !== "true") return null;
    return element.dataset.treePath ?? "";
  }

  function showDropOverlay(paths: string[]) {
    draggedPaths = paths;
    const destination = dropDestination(paths);
    dropLabel = destination ? dropMessage(destination, paths) : null;
  }

  function hideDropOverlay() {
    draggedPaths = [];
    dropLabel = null;
    treeDropTarget = null;
  }

  onMount(() => {
    const downloads = listen<DownloadProgress>("download://progress", (event) =>
      trackDownload(event.payload),
    );

    const pending = getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "enter") {
        showDropOverlay(event.payload.paths);
      } else if (event.payload.type === "over") {
        if (!dropLabel && draggedPaths.length > 0) showDropOverlay(draggedPaths);
        treeDropTarget =
          app.view === "editor" && !app.target?.standalone
            ? folderUnderPointer(
                event.payload.position.x,
                event.payload.position.y,
              )
            : null;
      } else if (event.payload.type === "drop") {
        const folder = treeDropTarget;
        hideDropOverlay();
        dropPaths(event.payload.paths, folder);
      } else {
        hideDropOverlay();
      }
    });

    registerHotkey("save", (event) => {
      event.preventDefault();
      if (app.view === "editor") saveAndCompile();
    });
    registerHotkey("undo", (event) => {
      event.preventDefault();
      if (app.view === "editor") undoEdit(editorView);
    });
    registerHotkey("redo", (event) => {
      event.preventDefault();
      if (app.view === "editor") redoEdit(editorView);
    });
    registerHotkey("toggleSidebar", (event) => {
      event.preventDefault();
      if (app.view === "editor" && !app.target?.standalone) toggleSidebar();
    });
    for (const commandId of formattingCommandIds) {
      registerHotkey(commandId, (event) => {
        event.preventDefault();
        if (app.view !== "editor") return;
        const language = isLatexPath(app.activePath ?? "") ? "latex" : "typst";
        runFormattingCommand(editorView, commandId, language);
      });
    }

    return () => {
      pending.then((unlisten) => unlisten());
      downloads.then((unlisten) => unlisten());
      unregisterAll();
    };
  });


  const wsLabel: Record<string, string> = {
    connected: "Live sync",
    connecting: "Connecting…",
    offline: "Offline (polling)",
  };

  const collabLabel: Record<string, string> = {
    connected: "Live",
    connecting: "Connecting…",
    offline: "Offline — editing locally",
  };
</script>


{#snippet statusBadge(
  icon: string,
  label: string,
  tone: "success" | "accent" | "muted",
  spin = false,
)}
  <span
    class="chip
      {tone === 'success'
      ? 'bg-[var(--color-success)]/12 text-[var(--color-success)]'
      : tone === 'accent'
        ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
        : 'bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]'}"
  >
    <Icon {icon} class="text-xs {spin ? 'animate-spin' : ''}" />
    {label}
  </span>
{/snippet}

<div class="flex h-screen flex-col">
  <header
    data-tauri-drag-region
    class="flex h-11 shrink-0 select-none items-center gap-1 border-b border-[var(--color-line)] bg-[var(--color-surface)] pl-2"
  >
    {#if app.view === "editor"}
      <button
        class="btn btn-ghost px-2"
        onclick={closeTarget}
      >
        <Icon icon="ph:arrow-left" />
        Files
      </button>

      {#if editorView}
        <button
          class="icon-btn"
          title="Undo"
          aria-label="Undo"
          onclick={() => undoEdit(editorView)}
        >
          <Icon icon="ph:arrow-counter-clockwise" class="text-sm" />
        </button>
        <button
          class="icon-btn"
          title="Redo"
          aria-label="Redo"
          onclick={() => redoEdit(editorView)}
        >
          <Icon icon="ph:arrow-clockwise" class="text-sm" />
        </button>
      {/if}

      <div class="divider-v"></div>
      <span data-tauri-drag-region class="truncate px-1 text-[13px] font-medium">
        {app.target?.path.split("/").pop()}
      </span>
      {#if app.target?.standalone}
        {@render statusBadge("ph:file", "Single file", "muted")}
      {/if}
      {#if app.dirty}
        <span class="h-1.5 w-1.5 shrink-0 rounded-full bg-[var(--color-accent)]" title="Unsaved changes"></span>
      {/if}
    {:else}
      <img src="/favicon.png" alt="Typst Desktop" class="ml-1 h-5 w-5 rounded-md" />
      <span data-tauri-drag-region class="ml-1 text-[13px] font-semibold tracking-tight">
        Typst Desktop
      </span>
    {/if}

    <div data-tauri-drag-region class="h-full flex-1"></div>

    {#if app.view === "editor"}
      {#if app.account && (app.target?.cloud_project_id || app.documentLink) && !app.collabIntent}
        <span title="Cloud sync connection">
          {@render statusBadge(
            app.wsStatus === "connected"
              ? "ph:cloud-check"
              : app.wsStatus === "connecting"
                ? "ph:circle-notch"
                : "ph:cloud-slash",
            wsLabel[app.wsStatus] ?? "Offline (polling)",
            app.wsStatus === "connected"
              ? "success"
              : app.wsStatus === "connecting"
                ? "accent"
                : "muted",
            app.wsStatus === "connecting",
          )}
        </span>
      {/if}

      {#if app.collabIntent}
        <span title="Realtime cloud sync">
          {@render statusBadge(
            app.collabStatus === "connected"
              ? "ph:broadcast"
              : app.collabStatus === "connecting"
                ? "ph:circle-notch"
                : "ph:wifi-slash",
            collabLabel[app.collabStatus ?? "offline"],
            app.collabStatus === "connected"
              ? "success"
              : app.collabStatus === "connecting"
                ? "accent"
                : "muted",
            app.collabStatus === "connecting",
          )}
        </span>
      {/if}

      {#if !app.collabIntent}
        <button
          class="btn btn-ghost"
          onclick={saveAndCompile}
        >
          <Icon icon="ph:floppy-disk" />
          Save
        </button>
      {/if}

      <PreviewModeMenu />

      <ExportMenu onexport={exportAs} />

      <div class="group relative">
        <button
          class="btn btn-ghost"
        >
          <Icon icon="ph:copy" />
          Copy
        </button>
        <div
          class="menu invisible absolute right-0 top-full z-20 w-32 opacity-0 transition-opacity group-hover:visible group-hover:opacity-100"
        >
          {#each ["png", "svg"] as format}
            <button
              class="menu-item uppercase"
              onclick={() => copyAs(format)}
            >
              {format}
            </button>
          {/each}
        </div>
      </div>

      {#if (app.target?.cloud_project_id || app.documentLink) && !app.collabIntent}
        <button
          class="btn btn-primary ml-1"
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
      class="icon-btn"
      onclick={() => (dialog = { kind: "settings" })}
      aria-label="Settings"
    >
      <Icon icon="ph:gear-six" />
    </button>

    <div class="divider-v ml-1 mr-0"></div>

    <WindowControls />
  </header>

  {#if app.download}
    {@const progress = app.download}
    <div
      class="flex shrink-0 items-center gap-3 border-b border-[var(--color-line)] bg-[var(--color-surface)] px-4 py-2"
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
        class="h-1 min-w-0 flex-1 overflow-hidden rounded-full bg-[var(--color-surface-sunken)]"
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
      class="flex shrink-0 items-center gap-2 border-b px-4 py-1.5 text-xs
        {app.error
        ? 'border-[var(--color-danger)]/25 bg-[var(--color-danger)]/8 text-[var(--color-danger)]'
        : 'border-[var(--color-success)]/25 bg-[var(--color-success)]/8 text-[var(--color-success)]'}"
    >
      <Icon icon={app.error ? "ph:warning-circle" : "ph:check-circle"} />
      <span class="flex-1">{app.error || app.status}</span>
      {#if app.conflicts.length > 0}
        <button
          class="btn h-6 border-current px-2"
          onclick={() => (dialog = { kind: "conflicts" })}
        >
          Resolve
        </button>
      {/if}
      <button class="icon-btn icon-btn-sm text-current hover:bg-current/10 hover:text-current" onclick={clearMessages} aria-label="Dismiss">
        <Icon icon="ph:x" />
      </button>
    </div>
  {/if}

  <div class="flex min-h-0 flex-1">
    {#if app.view === "files"}
      <BrowserSidebar onsettings={() => (dialog = { kind: "settings" })} />
      <div class="min-w-0 flex-1">
        {#if app.scope === "local"}
          <FileViewer
            onnewproject={() => (dialog = { kind: "new-project" })}
            onnewfolder={() => (dialog = { kind: "new-folder" })}
            onnewdocument={() => (dialog = { kind: "new-document" })}
            onupload={importFiles}
            onconvert={convertDocuments}
            onrename={(entry) => (dialog = { kind: "rename-entry", entry })}
            ondelete={(entry) => (dialog = { kind: "delete-entry", entry })}
            onlink={(entry) => (dialog = { kind: "link-entry", entry })}
            onsavetocloud={(entry) =>
              (dialog = { kind: "save-document-to-cloud", entry })}
            onviewfile={(paths, index) => (fileViewer = { paths, index })}
          />
        {:else}
          <CloudView
            ondownloaddocument={(documentId, title) =>
              downloadDocument(documentId, title)}
            onremovedownload={removeDownloadedDocument}
            ondownloadfile={downloadCloudFile}
            onopenfile={openCloudFile}
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
        {/if}
      </div>
    {:else}
      {#if !app.target?.standalone && sidebarCollapsed}
        <div
          class="flex w-10 shrink-0 flex-col items-center border-r border-[var(--color-line)] bg-[var(--color-surface)] py-1.5"
        >
          <button
            class="icon-btn"
            title="Show files"
            aria-label="Show files"
            onclick={toggleSidebar}
          >
            <Icon icon="ph:sidebar-simple" class="text-base" />
          </button>
        </div>
      {:else if !app.target?.standalone}
        <div
          class="flex w-60 shrink-0 flex-col border-r border-[var(--color-line)] bg-[var(--color-surface)]"
        >
          <div class="bar justify-between pl-3 pr-1.5">
            <span class="section-label truncate">
              {selectedFolder ? selectedFolder : "Files"}
            </span>
            <button
              class="icon-btn"
              title="Hide files"
              aria-label="Hide files"
              onclick={toggleSidebar}
            >
              <Icon icon="ph:sidebar-simple" class="text-sm" />
            </button>
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
        <div class="min-w-0 flex-1 flex-col border-r border-[var(--color-line)] {previewMode.current === 'preview' ? 'hidden' : 'flex'}">
          {#if app.activePath && activeFile?.is_text}
            <EditorToolbar
              view={editorView}
              language={isLatexPath(app.activePath) ? "latex" : "typst"}
              disabled={app.activePath.toLowerCase().endsWith(".toml")}
              onassets={() => (dialog = { kind: "assets" })}
              onpagesettings={() => (dialog = { kind: "page-settings" })}
            />

            <div
              class="flex h-9 shrink-0 items-center gap-2 border-b border-[var(--color-line)] bg-[var(--color-editor)] px-4 text-xs"
            >
              <Icon icon="ph:file-text" class="text-sm text-[var(--color-ink-subtle)]" />
              <span class="truncate font-medium">{app.activePath}</span>
              {#if app.dirty}
                <span class="chip h-5 bg-[var(--color-surface-sunken)] px-1.5 text-[10px] text-[var(--color-ink-muted)]">Edited</span>
              {/if}
              <ReviewBar />
            </div>

            {#key app.target?.path + ":" + app.activePath}
              <div class="min-h-0 flex-1">
                <Editor
                  content={app.editorContent}
                  filePath={app.activePath}
                  targetPath={app.target?.path ?? ""}
                  diagnostics={app.diagnostics}
                  extensions={editorExtensions}
                  collab={app.collab
                    ? { text: app.collab.text, awareness: app.collab.provider.awareness }
                    : null}
                  onchange={(value) => {
                    app.editorContent = value;
                    app.dirty = true;
                    scheduleCompile();
                    scheduleAutosave();
                  }}
                  onlspstatus={(status) => (app.lspStatus = status)}
                  onready={(view) => (editorView = view)}
                />
              </div>
              <ReviewLayer view={editorView} title={app.target?.path.split("/").pop() ?? app.activePath} />
            {/key}
          {:else if app.target && app.activePath && api.isViewablePath(app.activePath)}
            {@const viewPath = `${app.target.path}/${app.activePath}`}
            {#key viewPath}
              {#if api.isPdfPath(viewPath)}
                <PdfView path={viewPath} />
              {:else}
                <ImageView path={viewPath} />
              {/if}
            {/key}
          {:else}
            <div
              class="empty-state flex-1 bg-[var(--color-editor)]"
            >
              <Icon icon="ph:file-dashed" class="text-3xl" />
              <p class="text-sm">
                {app.activePath
                  ? "This file cannot be edited as text"
                  : "Select a file to edit"}
              </p>
            </div>
          {/if}
        </div>

        {#if previewMode.current !== "editor"}
          <div class="min-w-0 flex-1">
            <Preview
              compiled={app.compiled}
              diagnostics={app.diagnostics}
              compiling={app.compiling}
            />
          </div>
        {/if}

        <ReviewPanel />
      </div>
    {/if}
  </div>
</div>

{#if fileViewer}
  <FileViewerOverlay
    paths={fileViewer.paths}
    index={fileViewer.index}
    onclose={() => (fileViewer = null)}
  />
{/if}

{#if dropLabel}
  <div
    class="pointer-events-none fixed inset-0 z-40 flex items-center justify-center bg-[var(--backdrop)] p-8 backdrop-blur-[2px]"
  >
    <div
      class="flex flex-col items-center gap-2 rounded-2xl border-2 border-dashed border-[var(--color-accent)] bg-[var(--color-surface-raised)] px-12 py-9 shadow-2xl"
    >
      <span class="mb-1 flex h-12 w-12 items-center justify-center rounded-xl bg-[var(--color-accent-soft)]">
        <Icon icon="ph:tray-arrow-down" class="text-2xl text-[var(--color-accent)]" />
      </span>
      <p class="text-sm font-medium">{dropLabel}</p>
      <p class="text-xs text-[var(--color-ink-muted)]">
        Typst files, images, and fonts are copied in.
      </p>
    </div>
  </div>
{/if}

{#if dialog.kind === "new-project"}
  <PromptModal
    title="New {languageLabel(createLanguage)} project"
    label="Project name"
    icon="ph:folder-plus"
    placeholder="My thesis"
    onsubmit={createProject}
    onclose={close}
  >
    <LanguagePicker bind:language={createLanguage} />
  </PromptModal>
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
    title="New {languageLabel(createLanguage)} document"
    label="Document name"
    icon="ph:file-plus"
    placeholder="notes"
    suffix={sourceExtension(createLanguage)}
    onsubmit={createDocument}
    onclose={close}
  >
    <LanguagePicker bind:language={createLanguage} />
  </PromptModal>
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
    value={target.entry.name.replace(/\.(typ|tex)$/i, "")}
    confirmLabel="Save"
    onsubmit={(title) => saveDocumentToCloud(target.entry, title)}
    onclose={close}
  />
{:else if dialog.kind === "new-cloud-project"}
  <PromptModal
    title="New {languageLabel(createLanguage)} cloud project"
    label="Project name"
    icon="ph:cloud-plus"
    onsubmit={createCloudProject}
    onclose={close}
  >
    <LanguagePicker bind:language={createLanguage} />
  </PromptModal>
{:else if dialog.kind === "new-cloud-document"}
  <PromptModal
    title="New {languageLabel(createLanguage)} cloud document"
    label="Document title"
    icon="ph:cloud-plus"
    onsubmit={createCloudDocument}
    onclose={close}
  >
    <LanguagePicker bind:language={createLanguage} />
  </PromptModal>
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

{#if app.view === "editor" && app.target}
  <InsertDialogs
    view={editorView}
    targetPath={app.target.path}
    files={app.target.files}
    activePath={app.activePath}
  />
{/if}

{#if app.collabConflict}
  <ConflictModal
    conflicts={[app.collabConflict]}
    onresolve={(resolutions) => resolveCollabConflict(resolutions[0]?.content ?? "")}
    onclose={cancelCollabConflict}
  />
{/if}
