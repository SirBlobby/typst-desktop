import * as api from "$lib/ts/api";
import {
  app,
  compile,
  openFile,
  openTarget,
  refreshEntries,
  refreshTarget,
  setError,
  setStatus,
  shortcutForPath,
} from "$lib/ts/state.svelte";

export type DropDestination =
  | { kind: "project"; projectPath: string }
  | { kind: "workspace"; folder: string; refreshTarget: boolean };

export function isTypstPath(path: string): boolean {
  return path.toLowerCase().endsWith(".typ");
}

function parentFolder(path: string): string {
  const separator = path.lastIndexOf("/");
  return separator === -1 ? "" : path.slice(0, separator);
}

function joinPath(folder: string, name: string): string {
  return folder ? `${folder}/${name}` : name;
}

function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

function folderLabel(folder: string): string {
  const shortcut = shortcutForPath(folder);
  return shortcut && shortcut.browse_path === folder ? shortcut.name : fileName(folder);
}

export function dropDestination(paths: string[]): DropDestination | null {
  if (app.view === "editor" && app.target) {
    return app.target.standalone
      ? { kind: "workspace", folder: parentFolder(app.target.path), refreshTarget: true }
      : { kind: "project", projectPath: app.target.path };
  }
  if (app.view === "files" && app.scope === "local") {
    return { kind: "workspace", folder: app.currentDir, refreshTarget: false };
  }
  if (app.view === "files" && paths.some(isTypstPath)) {
    return { kind: "workspace", folder: "", refreshTarget: false };
  }
  return null;
}

export function dropMessage(destination: DropDestination, paths: string[]): string {
  const typstFiles = paths.filter(isTypstPath);
  const opensDocument = typstFiles.length === 1 && destination.kind === "workspace" && !destination.refreshTarget;

  if (opensDocument) return `Drop to open ${fileName(typstFiles[0])}`;
  if (destination.kind === "project") {
    return `Drop to add files to ${fileName(destination.projectPath)}`;
  }
  if (destination.folder) return `Drop to add files to ${folderLabel(destination.folder)}`;
  return "Drop to add files to your workspace";
}

async function dropIntoProject(projectPath: string, folder: string | null, paths: string[]) {
  const imported = await api.importIntoFolder(
    folder ? `${projectPath}/${folder}` : projectPath,
    paths,
  );
  await refreshTarget();

  const typstFiles = imported.filter(isTypstPath);
  if (typstFiles.length === 1) {
    await openFile(joinPath(folder ?? "", typstFiles[0]));
  } else {
    await compile();
  }
  return imported;
}

async function dropIntoWorkspace(destination: DropDestination & { kind: "workspace" }, paths: string[]) {
  const imported = await api.importIntoFolder(destination.folder, paths);

  if (destination.refreshTarget) {
    await refreshTarget();
    await compile();
    return imported;
  }

  await refreshEntries();
  const typstFiles = imported.filter(isTypstPath);
  if (typstFiles.length === 1) {
    await openTarget(joinPath(destination.folder, typstFiles[0]));
  }
  return imported;
}

export async function dropPaths(paths: string[], treeFolder: string | null) {
  const destination = dropDestination(paths);
  if (!destination || paths.length === 0) return;

  try {
    const imported =
      destination.kind === "project"
        ? await dropIntoProject(destination.projectPath, treeFolder, paths)
        : await dropIntoWorkspace(destination, paths);
    setStatus(`Imported ${imported.length} item(s)`);
  } catch (error) {
    setError(error);
  }
}
