<script lang="ts">
  import Icon from "@iconify/svelte";
  import * as api from "$lib/ts/api";
  import type { BrowseEntry, EntryKind } from "$lib/ts/api";
  import {
    app,
    breadcrumbs,
    browseTo,
    openTarget,
  } from "$lib/ts/state.svelte";

  interface Props {
    onnewfolder: () => void;
    onnewproject: () => void;
    onnewdocument: () => void;
    onupload: () => void;
    onassets: () => void;
    onrename: (entry: BrowseEntry) => void;
    ondelete: (entry: BrowseEntry) => void;
    onlink: (entry: BrowseEntry) => void;
    onviewimage: (paths: string[], index: number) => void;
    onclonespace: (spaceId: string, name: string) => void;
    ondeletespace: (spaceId: string) => void;
    onnewspace: () => void;
    onsignin: () => void;
  }

  let {
    onnewfolder,
    onnewproject,
    onnewdocument,
    onupload,
    onassets,
    onrename,
    ondelete,
    onlink,
    onviewimage,
    onclonespace,
    ondeletespace,
    onnewspace,
    onsignin,
  }: Props = $props();

  let menuFor = $state<string | null>(null);

  const trail = $derived(breadcrumbs());

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

  const localSpaceIds = $derived(
    new Set(
      app.entries
        .filter((entry) => entry.space_id)
        .map((entry) => entry.space_id as string),
    ),
  );

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

  const imagePaths = $derived(
    app.entries
      .filter((entry) => api.isImagePath(entry.path))
      .map((entry) => entry.path),
  );

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

{#snippet actions(entry: BrowseEntry, offset: string)}
  <button
    class="absolute right-2 {offset} rounded p-1 text-[var(--color-ink-muted)] opacity-0 transition group-hover:opacity-100 hover:bg-[var(--color-surface-sunken)]"
    onclick={() => (menuFor = menuFor === entry.path ? null : entry.path)}
    aria-label="Actions"
  >
    <Icon icon="ph:dots-three-vertical" />
  </button>

  {#if menuFor === entry.path}
    <div
      class="absolute right-2 top-9 z-10 flex w-40 flex-col rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] py-1 text-xs shadow-lg"
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
      {#if entry.kind === "project" && !entry.space_id && app.account}
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

<div class="flex h-full flex-col bg-[var(--color-surface-muted)]">
  <div
    class="flex items-center gap-2 border-b border-[var(--color-line)] bg-[var(--color-surface)] px-4 py-2.5"
  >
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

    <div class="flex-1"></div>

    {#if app.scope === "local"}
      <button
        class="flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)]"
        onclick={onassets}
      >
        <Icon icon="ph:images" />
        Assets
      </button>
      <button
        class="flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)]"
        onclick={onupload}
      >
        <Icon icon="ph:upload-simple" />
        Import
      </button>
      <button
        class="flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)]"
        onclick={onnewfolder}
      >
        <Icon icon="ph:folder-plus" />
        Folder
      </button>
      <button
        class="flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)]"
        onclick={onnewdocument}
      >
        <Icon icon="ph:file-plus" />
        Document
      </button>
      <button
        class="flex items-center gap-1.5 rounded-md bg-[var(--color-accent)] px-2.5 py-1.5 text-xs font-medium text-white transition hover:opacity-90"
        onclick={onnewproject}
      >
        <Icon icon="ph:plus" />
        New project
      </button>
    {:else if app.account}
      <button
        class="flex items-center gap-1.5 rounded-md bg-[var(--color-accent)] px-2.5 py-1.5 text-xs font-medium text-white transition hover:opacity-90"
        onclick={onnewspace}
      >
        <Icon icon="ph:plus" />
        New space
      </button>
    {/if}
  </div>

  {#if app.scope === "local"}
    <div
      class="flex items-center gap-1 border-b border-[var(--color-line)] bg-[var(--color-surface)] px-4 py-2 text-xs"
    >
      <button
        class="flex items-center gap-1.5 rounded px-2 py-1 transition hover:bg-[var(--color-surface-muted)]
          {app.currentDir === '' ? 'font-medium' : 'text-[var(--color-ink-muted)]'}"
        onclick={() => browseTo("")}
      >
        <Icon icon="ph:house" />
        Workspace
      </button>

      {#each trail as crumb, index}
        <Icon icon="ph:caret-right" class="text-[10px] text-[var(--color-ink-muted)]" />
        <button
          class="rounded px-2 py-1 transition hover:bg-[var(--color-surface-muted)]
            {index === trail.length - 1 ? 'font-medium' : 'text-[var(--color-ink-muted)]'}"
          onclick={() => browseTo(crumb.path)}
        >
          {crumb.name}
        </button>
      {/each}
    </div>

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
              onclick={onnewproject}
            >
              New project
            </button>
            <button
              class="rounded-md border border-[var(--color-line)] px-3 py-1.5 text-xs hover:bg-[var(--color-surface)]"
              onclick={onnewdocument}
            >
              New document
            </button>
          </div>
        </div>
      {:else}
        <div class="flex flex-col gap-5">
          {#if containers.length > 0}
            <section class="flex flex-col gap-2">
              <h2
                class="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]"
              >
                Folders
              </h2>
              <div class="grid grid-cols-[repeat(auto-fill,minmax(210px,1fr))] gap-2">
                {#each containers as entry (entry.path)}
                  <div
                    class="group relative flex items-center gap-2.5 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-sunken)] px-3 py-2.5 transition hover:border-[var(--color-accent)] hover:bg-[var(--color-surface)]"
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
                          {#if entry.space_id}
                            <Icon
                              icon="ph:cloud-check"
                              class="shrink-0 text-xs text-[var(--color-success)]"
                            />
                          {/if}
                        </span>
                        <span
                          class="block truncate text-[10px] text-[var(--color-ink-muted)]"
                        >
                          {entry.kind === "project" ? "Project · " : ""}{entry.child_count}
                          items
                        </span>
                      </span>
                    </button>

                    {@render actions(entry, "top-2")}
                  </div>
                {/each}
              </div>
            </section>
          {/if}

          {#if documents.length > 0}
            <section class="flex flex-col gap-2">
              <h2
                class="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]"
              >
                Documents
              </h2>
              <div class="grid grid-cols-[repeat(auto-fill,minmax(150px,1fr))] gap-3">
                {#each documents as entry (entry.path)}
                  <div
                    class="group relative flex flex-col overflow-hidden rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] transition hover:border-[var(--color-accent)] hover:shadow-md"
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

                      <span class="flex flex-col gap-0.5 px-2.5 py-2">
                        <span
                          class="truncate text-xs font-medium"
                          title={entry.name}
                        >
                          {entry.name}
                        </span>
                        <span class="text-[10px] text-[var(--color-ink-muted)]">
                          {formatSize(entry.size)}
                          {#if entry.modified}
                            · {formatDate(entry.modified)}
                          {/if}
                        </span>
                      </span>
                    </button>

                    {@render actions(entry, "top-2")}
                  </div>
                {/each}
              </div>
            </section>
          {/if}
        </div>
      {/if}
    </div>
  {:else}
    <div class="scroll-thin flex-1 overflow-y-auto p-4">
      {#if !app.account}
        <div
          class="flex h-full flex-col items-center justify-center gap-3 text-[var(--color-ink-muted)]"
        >
          <Icon icon="ph:cloud-slash" class="text-5xl" />
          <p class="max-w-xs text-center text-sm">
            Connect a TypstDrive account to sync your projects across devices.
          </p>
          <button
            class="rounded-md bg-[var(--color-accent)] px-3 py-2 text-xs font-medium text-white hover:opacity-90"
            onclick={onsignin}
          >
            Sign in
          </button>
        </div>
      {:else if app.spaces.length === 0}
        <div
          class="flex h-full flex-col items-center justify-center gap-3 text-[var(--color-ink-muted)]"
        >
          <Icon icon="ph:cloud" class="text-5xl" />
          <p class="text-sm">No cloud spaces yet.</p>
        </div>
      {:else}
        <div class="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-3">
          {#each app.spaces as space (space.id)}
            <div
              class="group flex flex-col gap-2 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-3 transition hover:border-[var(--color-accent)]"
            >
              <div class="flex items-center gap-2">
                <Icon icon="ph:cloud" class="text-xl text-[var(--color-accent)]" />
                {#if localSpaceIds.has(space.id)}
                  <span title="Downloaded to this device" class="flex">
                    <Icon
                      icon="ph:hard-drives"
                      class="text-sm text-[var(--color-success)]"
                    />
                  </span>
                {/if}
              </div>

              <span class="truncate text-xs font-medium">{space.name}</span>
              <span class="text-[10px] text-[var(--color-ink-muted)]">
                {space.role} · {formatDate(space.updated_at)}
              </span>

              <div class="mt-1 flex gap-1">
                {#if !localSpaceIds.has(space.id)}
                  <button
                    class="flex flex-1 items-center justify-center gap-1 rounded border border-[var(--color-line)] px-2 py-1 text-[10px] hover:bg-[var(--color-surface-muted)]"
                    onclick={() => onclonespace(space.id, space.name)}
                  >
                    <Icon icon="ph:download-simple" />
                    Download
                  </button>
                {/if}
                {#if space.role === "owner"}
                  <button
                    class="rounded border border-[var(--color-line)] px-2 py-1 text-[10px] text-[var(--color-danger)] hover:bg-[var(--color-surface-muted)]"
                    onclick={() => ondeletespace(space.id)}
                    aria-label="Delete space"
                  >
                    <Icon icon="ph:trash" />
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>
