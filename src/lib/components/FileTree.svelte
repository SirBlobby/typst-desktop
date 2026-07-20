<script lang="ts">
  import Icon from "@iconify/svelte";
  import { resolveResource } from "@tauri-apps/api/path";
  import { startDrag } from "@crabnebula/tauri-plugin-drag";
  import { absolutePath, type FileEntry } from "$lib/ts/api";

  interface Props {
    files: FileEntry[];
    activePath: string | null;
    entrypoint: string;
    targetPath: string;
    selected: Set<string>;
    dropTarget: string | null;
    onopen: (path: string) => void;
    onselect: (paths: string[], primary: string | null, isDir: boolean) => void;
    onrename: (path: string) => void;
    ondelete: (paths: string[]) => void;
    onduplicate: (path: string) => void;
    onreveal: (path: string) => void;
    onsetentry: (path: string) => void;
    onmove: (path: string, destination: string) => void;
    onnewfile: (parent: string) => void;
    onnewfolder: (parent: string) => void;
    onimport: (parent: string) => void;
  }

  let {
    files,
    activePath,
    entrypoint,
    targetPath,
    selected,
    dropTarget,
    onopen,
    onselect,
    onrename,
    ondelete,
    onduplicate,
    onreveal,
    onsetentry,
    onmove,
    onnewfile,
    onnewfolder,
    onimport,
  }: Props = $props();

  interface TreeNode {
    name: string;
    path: string;
    file: FileEntry | null;
    isDir: boolean;
    children: TreeNode[];
  }

  const tree = $derived(buildTree(files));

  function buildTree(entries: FileEntry[]): TreeNode[] {
    const root: TreeNode = {
      name: "",
      path: "",
      file: null,
      isDir: true,
      children: [],
    };

    for (const entry of entries) {
      const segments = entry.path.split("/");
      let node = root;

      segments.forEach((segment, index) => {
        const path = segments.slice(0, index + 1).join("/");
        const isLeaf = index === segments.length - 1;
        let child = node.children.find((candidate) => candidate.name === segment);

        if (!child) {
          child = {
            name: segment,
            path,
            file: isLeaf && !entry.is_dir ? entry : null,
            isDir: isLeaf ? entry.is_dir : true,
            children: [],
          };
          node.children.push(child);
        } else if (isLeaf && !entry.is_dir) {
          child.file = entry;
          child.isDir = false;
        }

        node = child;
      });
    }

    return sortNodes(root.children);
  }

  function sortNodes(nodes: TreeNode[]): TreeNode[] {
    nodes.sort((a, b) => {
      if (a.isDir !== b.isDir) return a.isDir ? -1 : 1;
      return a.name.localeCompare(b.name);
    });
    for (const node of nodes) sortNodes(node.children);
    return nodes;
  }

  function iconFor(node: TreeNode): string {
    if (node.isDir) return "ph:folder";
    if (node.name.endsWith(".typ")) return "ph:file-text";
    if (node.name.endsWith(".toml")) return "ph:gear-six";
    if (node.file?.is_text) return "ph:file";
    return "ph:image";
  }

  let collapsed = $state<Record<string, boolean>>({});
  let menu = $state<{ path: string; isDir: boolean; x: number; y: number } | null>(
    null,
  );
  let dragging = $state<string | null>(null);
  let dragPaths = $state<string[]>([]);
  let anchorPath = $state<string | null>(null);
  let nativeDragSent = false;
  let iconPathPromise: Promise<string> | null = null;

  function flattenVisible(nodes: TreeNode[]): TreeNode[] {
    const result: TreeNode[] = [];
    for (const node of nodes) {
      result.push(node);
      if (node.isDir && node.children.length > 0 && !collapsed[node.path]) {
        result.push(...flattenVisible(node.children));
      }
    }
    return result;
  }

  function selectRange(from: string, to: TreeNode) {
    const flat = flattenVisible(tree);
    const fromIndex = flat.findIndex((node) => node.path === from);
    const toIndex = flat.findIndex((node) => node.path === to.path);

    if (fromIndex === -1 || toIndex === -1) {
      onselect([to.path], to.path, to.isDir);
      return;
    }

    const [start, end] =
      fromIndex < toIndex ? [fromIndex, toIndex] : [toIndex, fromIndex];
    const paths = flat.slice(start, end + 1).map((node) => node.path);
    onselect(paths, to.path, to.isDir);
  }

  function handleRowClick(event: MouseEvent, node: TreeNode) {
    if (event.shiftKey && anchorPath) {
      selectRange(anchorPath, node);
      return;
    }

    if (event.ctrlKey || event.metaKey) {
      const next = new Set(selected);
      if (next.has(node.path)) {
        next.delete(node.path);
      } else {
        next.add(node.path);
      }
      anchorPath = node.path;
      onselect([...next], node.path, node.isDir);
      return;
    }

    anchorPath = node.path;
    onselect([node.path], node.path, node.isDir);

    if (node.isDir) {
      collapsed[node.path] = !collapsed[node.path];
    } else {
      onopen(node.path);
    }
  }

  function openMenu(event: MouseEvent, node: TreeNode) {
    event.preventDefault();
    if (!selected.has(node.path)) {
      anchorPath = node.path;
      onselect([node.path], node.path, node.isDir);
    }
    menu = { path: node.path, isDir: node.isDir, x: event.clientX, y: event.clientY };
  }

  function handleKey(event: KeyboardEvent, node: TreeNode) {
    if (event.key === "F2") {
      event.preventDefault();
      onrename(node.path);
    }
    if (event.key === "Delete") {
      event.preventDefault();
      const paths =
        selected.has(node.path) && selected.size > 1 ? [...selected] : [node.path];
      ondelete(paths);
    }
  }

  function collapseAll() {
    const next: Record<string, boolean> = {};
    for (const entry of files) {
      if (entry.is_dir) next[entry.path] = true;
    }
    collapsed = next;
  }

  export function expandTo(path: string) {
    const segments = path.split("/");
    for (let index = 1; index < segments.length; index += 1) {
      collapsed[segments.slice(0, index).join("/")] = false;
    }
  }

  function parentOf(path: string): string {
    const index = path.lastIndexOf("/");
    return index === -1 ? "" : path.slice(0, index);
  }

  function joinTargetPath(path: string): string {
    return targetPath ? `${targetPath}/${path}` : path;
  }

  function resolveAbsolutePaths(paths: string[]): Promise<string[]> {
    return Promise.all(paths.map((path) => absolutePath(joinTargetPath(path))));
  }

  function dragIcon(): Promise<string> {
    if (!iconPathPromise) iconPathPromise = resolveResource("icons/32x32.png");
    return iconPathPromise;
  }

  async function exportViaOsDrag(paths: string[]) {
    if (nativeDragSent || paths.length === 0) return;
    nativeDragSent = true;
    try {
      const [absolutePaths, icon] = await Promise.all([
        resolveAbsolutePaths(paths),
        dragIcon(),
      ]);
      await startDrag({ item: absolutePaths, icon });
    } catch (error) {
      console.error("Failed to start native drag", error);
    }
  }

  function handleWindowDragLeave(event: DragEvent) {
    if (!dragging || nativeDragSent) return;
    // relatedTarget is null only when the pointer leaves the whole window,
    // as opposed to moving between elements inside it.
    if (event.relatedTarget !== null) return;
    exportViaOsDrag(dragPaths);
  }
</script>

<svelte:window
  on:click={() => (menu = null)}
  on:contextmenu={(event) => {
    if (!(event.target as HTMLElement).closest("[data-tree-row]")) menu = null;
  }}
  on:dragleave={handleWindowDragLeave}
/>

{#snippet branch(nodes: TreeNode[], depth: number)}
  {#each nodes as node (node.path)}
    <div>
      <div
        data-tree-row
        data-tree-path={node.path}
        data-tree-dir={node.isDir ? "true" : "false"}
        role="treeitem"
        tabindex="0"
        aria-selected={selected.has(node.path)}
        draggable="true"
        class="group flex items-center gap-1.5 rounded px-2 py-1 text-xs transition
          {node.path === activePath
          ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
          : selected.has(node.path)
            ? 'bg-[var(--color-surface-sunken)]'
            : 'text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]'}
          {dropTarget === node.path
          ? 'ring-1 ring-inset ring-[var(--color-accent)]'
          : ''}"
        style="padding-left: {depth * 12 + 8}px"
        onclick={(event) => handleRowClick(event, node)}
        oncontextmenu={(event) => openMenu(event, node)}
        onkeydown={(event) => handleKey(event, node)}
        ondragstart={(event) => {
          dragging = node.path;
          dragPaths =
            selected.has(node.path) && selected.size > 1
              ? [...selected]
              : [node.path];
          nativeDragSent = false;
          event.dataTransfer?.setData("text/plain", node.path);
        }}
        ondragend={() => {
          dragging = null;
          dragPaths = [];
          nativeDragSent = false;
        }}
        ondragover={(event) => {
          if (!dragging || !node.isDir) return;
          event.preventDefault();
        }}
        ondrop={(event) => {
          event.preventDefault();
          const source = dragging ?? event.dataTransfer?.getData("text/plain");
          dragging = null;
          if (!source) return;
          const destination = node.isDir ? node.path : parentOf(node.path);
          if (parentOf(source) === destination) return;
          onmove(source, destination);
        }}
      >
        {#if node.isDir}
          <Icon
            icon={collapsed[node.path] ? "ph:caret-right" : "ph:caret-down"}
            class="shrink-0 text-[10px] text-[var(--color-ink-muted)]"
          />
        {:else}
          <span class="w-2.5 shrink-0"></span>
        {/if}

        <Icon icon={iconFor(node)} class="shrink-0 text-sm" />
        <span class="min-w-0 flex-1 truncate">{node.name}</span>

        {#if node.path === entrypoint}
          <span
            class="shrink-0 rounded bg-[var(--color-accent)] px-1 py-px text-[9px] font-medium text-white"
          >
            main
          </span>
        {/if}

        <button
          class="shrink-0 rounded p-0.5 text-[var(--color-ink-muted)] opacity-0 transition group-hover:opacity-100 hover:bg-[var(--color-surface)]"
          onclick={(event) => {
            event.stopPropagation();
            openMenu(event, node);
          }}
          aria-label="Actions"
        >
          <Icon icon="ph:dots-three-vertical" />
        </button>
      </div>

      {#if node.children.length > 0 && !collapsed[node.path]}
        {@render branch(node.children, depth + 1)}
      {/if}
    </div>
  {/each}
{/snippet}

<div class="flex min-h-0 flex-1 flex-col">
  <div
    class="flex items-center justify-end gap-0.5 border-b border-[var(--color-line)] px-2 py-1"
  >
    <button
      class="rounded p-1 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]"
      onclick={() => onnewfile(anchorPath ?? "")}
      title="New file"
      aria-label="New file"
    >
      <Icon icon="ph:file-plus" />
    </button>
    <button
      class="rounded p-1 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]"
      onclick={() => onnewfolder(anchorPath ?? "")}
      title="New folder"
      aria-label="New folder"
    >
      <Icon icon="ph:folder-plus" />
    </button>
    <button
      class="rounded p-1 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]"
      onclick={() => onimport(anchorPath ?? "")}
      title="Import files"
      aria-label="Import files"
    >
      <Icon icon="ph:upload-simple" />
    </button>
    <button
      class="rounded p-1 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]"
      onclick={collapseAll}
      title="Collapse all"
      aria-label="Collapse all"
    >
      <Icon icon="ph:arrows-in-line-vertical" />
    </button>
  </div>

  <div
    class="scroll-thin flex-1 overflow-y-auto py-1"
    role="tree"
    tabindex="-1"
    data-tree-path=""
    data-tree-dir="true"
    onclick={(event) => {
      if (event.target === event.currentTarget) {
        anchorPath = null;
        onselect([], null, true);
      }
    }}
    onkeydown={(event) => {
      if (event.key === "Escape") {
        anchorPath = null;
        onselect([], null, true);
      }
    }}
    ondragover={(event) => {
      if (dragging) event.preventDefault();
    }}
    ondrop={(event) => {
      event.preventDefault();
      const source = dragging ?? event.dataTransfer?.getData("text/plain");
      dragging = null;
      if (source && parentOf(source) !== "") onmove(source, "");
    }}
  >
    {#if files.length === 0}
      <p class="px-3 py-4 text-xs text-[var(--color-ink-muted)]">No files yet</p>
    {:else}
      {@render branch(tree, 0)}
    {/if}
  </div>
</div>

{#if menu}
  {@const target = menu}
  {@const menuPaths =
    selected.has(target.path) && selected.size > 1 ? [...selected] : [target.path]}
  {@const single = menuPaths.length === 1}
  <div
    class="fixed z-50 flex w-48 flex-col rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] py-1 text-xs shadow-lg"
    style="left: {target.x}px; top: {target.y}px"
  >
    {#if single && target.isDir}
      <button
        class="px-3 py-1.5 text-left hover:bg-[var(--color-surface-sunken)]"
        onclick={() => onnewfile(target.path)}
      >
        New file
      </button>
      <button
        class="px-3 py-1.5 text-left hover:bg-[var(--color-surface-sunken)]"
        onclick={() => onnewfolder(target.path)}
      >
        New folder
      </button>
      <button
        class="px-3 py-1.5 text-left hover:bg-[var(--color-surface-sunken)]"
        onclick={() => onimport(target.path)}
      >
        Import files here
      </button>
      <div class="my-1 h-px bg-[var(--color-line)]"></div>
    {:else if single && target.path.endsWith(".typ")}
      <button
        class="px-3 py-1.5 text-left hover:bg-[var(--color-surface-sunken)]"
        onclick={() => onsetentry(target.path)}
      >
        Set as entrypoint
      </button>
      <div class="my-1 h-px bg-[var(--color-line)]"></div>
    {/if}

    {#if single}
      <button
        class="px-3 py-1.5 text-left hover:bg-[var(--color-surface-sunken)]"
        onclick={() => onrename(target.path)}
      >
        Rename
      </button>
      <button
        class="px-3 py-1.5 text-left hover:bg-[var(--color-surface-sunken)]"
        onclick={() => onduplicate(target.path)}
      >
        Duplicate
      </button>
      <button
        class="px-3 py-1.5 text-left hover:bg-[var(--color-surface-sunken)]"
        onclick={() => navigator.clipboard.writeText(target.path)}
      >
        Copy path
      </button>
      <button
        class="px-3 py-1.5 text-left hover:bg-[var(--color-surface-sunken)]"
        onclick={() => onreveal(target.path)}
      >
        Reveal in file manager
      </button>
      <div class="my-1 h-px bg-[var(--color-line)]"></div>
    {/if}

    <button
      class="px-3 py-1.5 text-left text-[var(--color-danger)] hover:bg-[var(--color-surface-sunken)]"
      onclick={() => ondelete(menuPaths)}
    >
      {single ? "Delete" : `Delete ${menuPaths.length} items`}
    </button>
  </div>
{/if}
