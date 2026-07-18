<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { FileEntry } from "$lib/ts/api";

  interface Props {
    files: FileEntry[];
    activePath: string | null;
    entrypoint: string;
    onopen: (path: string) => void;
    onrename: (path: string) => void;
    ondelete: (path: string) => void;
    onsetentry: (path: string) => void;
  }

  let {
    files,
    activePath,
    entrypoint,
    onopen,
    onrename,
    ondelete,
    onsetentry,
  }: Props = $props();

  interface TreeNode {
    name: string;
    path: string;
    file: FileEntry | null;
    children: TreeNode[];
  }

  const tree = $derived(buildTree(files));

  function buildTree(entries: FileEntry[]): TreeNode[] {
    const root: TreeNode = { name: "", path: "", file: null, children: [] };

    for (const entry of entries) {
      const segments = entry.path.split("/");
      let node = root;

      segments.forEach((segment, index) => {
        const path = segments.slice(0, index + 1).join("/");
        const isLeaf = index === segments.length - 1;
        let child = node.children.find((c) => c.name === segment);

        if (!child) {
          child = {
            name: segment,
            path,
            file: isLeaf ? entry : null,
            children: [],
          };
          node.children.push(child);
        }
        node = child;
      });
    }

    return sortNodes(root.children);
  }

  function sortNodes(nodes: TreeNode[]): TreeNode[] {
    nodes.sort((a, b) => {
      const aIsFolder = a.file === null;
      const bIsFolder = b.file === null;
      if (aIsFolder !== bIsFolder) return aIsFolder ? -1 : 1;
      return a.name.localeCompare(b.name);
    });
    for (const node of nodes) sortNodes(node.children);
    return nodes;
  }

  function iconFor(node: TreeNode): string {
    if (!node.file) return "ph:folder";
    if (node.name.endsWith(".typ")) return "ph:file-text";
    if (node.name.endsWith(".toml")) return "ph:gear-six";
    if (node.file.is_text) return "ph:file";
    return "ph:image";
  }

  let collapsed = $state<Record<string, boolean>>({});
  let menuPath = $state<string | null>(null);
</script>

{#snippet branch(nodes: TreeNode[], depth: number)}
  {#each nodes as node (node.path)}
    <div>
      <div
        class="group flex items-center gap-1.5 rounded px-2 py-1 text-xs transition
          {node.path === activePath
          ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
          : 'text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]'}"
        style="padding-left: {depth * 12 + 8}px"
      >
        <button
          class="flex min-w-0 flex-1 items-center gap-1.5 text-left"
          onclick={() => {
            if (node.file) {
              onopen(node.path);
            } else {
              collapsed[node.path] = !collapsed[node.path];
            }
          }}
        >
          {#if !node.file}
            <Icon
              icon={collapsed[node.path] ? "ph:caret-right" : "ph:caret-down"}
              class="shrink-0 text-[10px] text-[var(--color-ink-muted)]"
            />
          {/if}
          <Icon icon={iconFor(node)} class="shrink-0 text-sm" />
          <span class="truncate">{node.name}</span>
          {#if node.path === entrypoint}
            <span
              class="shrink-0 rounded bg-[var(--color-accent)] px-1 py-px text-[9px] font-medium text-white"
            >
              main
            </span>
          {/if}
        </button>

        {#if node.file}
          <button
            class="shrink-0 rounded p-0.5 text-[var(--color-ink-muted)] opacity-0 transition group-hover:opacity-100 hover:bg-[var(--color-surface)]"
            onclick={() => (menuPath = menuPath === node.path ? null : node.path)}
            aria-label="File actions"
          >
            <Icon icon="ph:dots-three-vertical" />
          </button>
        {/if}
      </div>

      {#if node.file && menuPath === node.path}
        <div
          class="ml-6 mb-1 flex flex-col rounded border border-[var(--color-line)] bg-[var(--color-surface)] py-1 text-xs shadow-sm"
        >
          {#if node.name.endsWith(".typ")}
            <button
              class="px-3 py-1.5 text-left hover:bg-[var(--color-surface-sunken)]"
              onclick={() => {
                onsetentry(node.path);
                menuPath = null;
              }}
            >
              Set as entrypoint
            </button>
          {/if}
          <button
            class="px-3 py-1.5 text-left hover:bg-[var(--color-surface-sunken)]"
            onclick={() => {
              onrename(node.path);
              menuPath = null;
            }}
          >
            Rename
          </button>
          <button
            class="px-3 py-1.5 text-left text-[var(--color-danger)] hover:bg-[var(--color-surface-sunken)]"
            onclick={() => {
              ondelete(node.path);
              menuPath = null;
            }}
          >
            Delete
          </button>
        </div>
      {/if}

      {#if node.children.length > 0 && !collapsed[node.path]}
        {@render branch(node.children, depth + 1)}
      {/if}
    </div>
  {/each}
{/snippet}

<div class="scroll-thin flex-1 overflow-y-auto py-1">
  {#if files.length === 0}
    <p class="px-3 py-4 text-xs text-[var(--color-ink-muted)]">No files yet</p>
  {:else}
    {@render branch(tree, 0)}
  {/if}
</div>
