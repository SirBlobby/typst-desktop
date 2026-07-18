<script lang="ts">
  import Icon from "@iconify/svelte";
  import * as api from "$lib/ts/api";
  import type { BrowseEntry, EntryKind } from "$lib/ts/api";
  import {
    app,
    breadcrumbs,
    browseTo,
    cloudBreadcrumbs,
    linkedDocument,
    linkedSpace,
    openCloudFolder,
    openTarget,
    refreshCloud,
  } from "$lib/ts/state.svelte";

  interface Props {
    onnewfolder: () => void;
    onnewproject: () => void;
    onnewdocument: () => void;
    onupload: () => void;
    onrename: (entry: BrowseEntry) => void;
    ondelete: (entry: BrowseEntry) => void;
    onlink: (entry: BrowseEntry) => void;
    onviewimage: (paths: string[], index: number) => void;
    ondownloaddocument: (documentId: string, title: string) => void;
    onremovedownload: (path: string) => void;
    ondownloadfile: (fileId: string, name: string) => void;
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
    onrename,
    ondelete,
    onlink,
    onviewimage,
    ondownloaddocument,
    onremovedownload,
    ondownloadfile,
    onclonespace,
    ondeletespace,
    onnewspace,
    onsignin,
  }: Props = $props();

  let menuFor = $state<string | null>(null);
  let menuAt = $state({ x: 0, y: 0 });

  const trail = $derived(breadcrumbs());
  const cloudTrail = $derived(cloudBreadcrumbs());

  $effect(() => {
    if (app.scope === "cloud" && app.account) {
      refreshCloud();
    }
  });

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
  let cloudThumbs = $state<Record<string, { kind: string; data: string }>>({});

  $effect(() => {
    if (app.scope !== "cloud") return;

    const pending = [
      ...app.linkedDocuments.map((linked) => linked.path),
      ...app.linkedSpaces.map((linked) => linked.path),
    ];
    let cancelled = false;

    (async () => {
      for (const path of pending) {
        if (cancelled) return;
        if (cloudThumbs[path]) continue;
        try {
          const result = await api.thumbnail(path);
          if (!cancelled) cloudThumbs[path] = result;
        } catch {
          continue;
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  });

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

{#snippet cloudCard(
  icon: string,
  title: string,
  meta: string,
  link: { sync_state: string | null; path: string } | undefined,
  onopen: (() => void) | null,
  ondownload: () => void,
  onremove: (() => void) | null,
)}
  <div
    class="group flex flex-col overflow-hidden rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] transition hover:border-[var(--color-accent)] hover:shadow-sm"
  >
    <div
      class="flex h-24 items-center justify-center overflow-hidden border-b border-[var(--color-line)] bg-[var(--color-surface-muted)]"
    >
      {#if link && cloudThumbs[link.path]}
        {#if cloudThumbs[link.path].kind === "svg"}
          <span
            class="flex w-full items-start justify-center bg-white p-1 [&_svg]:h-auto [&_svg]:w-full"
          >
            {@html cloudThumbs[link.path].data}
          </span>
        {:else}
          <img
            src={cloudThumbs[link.path].data}
            alt={title}
            class="h-full w-full object-contain"
          />
        {/if}
      {:else}
        <Icon
          {icon}
          class="text-3xl {link
            ? 'text-[var(--color-accent)]'
            : 'text-[var(--color-ink-muted)]'}"
        />
      {/if}
    </div>

    <div class="flex flex-col gap-2.5 p-3">
    <div class="flex items-start gap-2">
      <div class="min-w-0 flex-1">
        <p class="truncate text-xs font-medium" {title}>{title}</p>
        <p class="truncate text-[10px] text-[var(--color-ink-muted)]">{meta}</p>
      </div>

      {#if link}
        <span
          class="flex shrink-0 items-center gap-1 rounded-full px-1.5 py-0.5 text-[9px] font-medium
            {link.sync_state === 'pending'
            ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
            : 'bg-[var(--color-success)]/10 text-[var(--color-success)]'}"
          title={link.sync_state === "pending"
            ? "Local changes not yet synced"
            : "On this device and up to date"}
        >
          <Icon
            icon={link.sync_state === "pending"
              ? "ph:cloud-arrow-up"
              : "ph:cloud-check"}
            class="text-[11px]"
          />
          {link.sync_state === "pending" ? "Unsynced" : "Synced"}
        </span>
      {/if}
    </div>

    <div class="flex gap-1">
      {#if link && onopen}
        <button
          class="flex flex-1 items-center justify-center gap-1 rounded-md bg-[var(--color-accent)] px-2 py-1.5 text-[10px] font-medium text-white transition hover:opacity-90"
          onclick={onopen}
        >
          <Icon icon="ph:pencil-simple" />
          Open
        </button>
      {:else}
        <button
          class="flex flex-1 items-center justify-center gap-1 rounded-md border border-[var(--color-line)] px-2 py-1.5 text-[10px] transition hover:bg-[var(--color-surface-muted)]"
          onclick={ondownload}
        >
          <Icon icon="ph:download-simple" />
          Download
        </button>
      {/if}

      {#if onremove}
        <button
          class="rounded-md border border-[var(--color-line)] px-2 py-1.5 text-[10px] text-[var(--color-ink-muted)] transition hover:border-[var(--color-danger)] hover:text-[var(--color-danger)]"
          onclick={onremove}
          aria-label="Remove"
        >
          <Icon icon="ph:trash" />
        </button>
      {/if}
    </div>
    </div>
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
      class="fixed z-50 flex w-40 -translate-x-full flex-col rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] py-1 text-xs shadow-lg"
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

<svelte:window
  on:click={(event) => {
    if (!(event.target as HTMLElement).closest("[data-card-menu]")) {
      menuFor = null;
    }
  }}
/>

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
      {:else}
        <div class="mb-4 flex items-center gap-2">
          <button
            class="flex items-center gap-2 rounded-lg border px-3.5 py-2 text-xs font-medium transition
              {app.cloudFolder !== 'shared'
              ? 'border-[var(--color-accent)] bg-[var(--color-accent)] text-white shadow-sm'
              : 'border-[var(--color-line)] bg-[var(--color-surface)] text-[var(--color-ink-muted)] hover:border-[var(--color-accent)] hover:text-[var(--color-ink)]'}"
            onclick={() => openCloudFolder(null)}
          >
            <Icon icon="ph:cloud-fill" class="text-base" />
            My Drive
          </button>

          <button
            class="flex items-center gap-2 rounded-lg border px-3.5 py-2 text-xs font-medium transition
              {app.cloudFolder === 'shared'
              ? 'border-[var(--color-accent)] bg-[var(--color-accent)] text-white shadow-sm'
              : 'border-[var(--color-line)] bg-[var(--color-surface)] text-[var(--color-ink-muted)] hover:border-[var(--color-accent)] hover:text-[var(--color-ink)]'}"
            onclick={() => openCloudFolder("shared")}
          >
            <Icon icon="ph:users-three-fill" class="text-base" />
            Shared with me
          </button>

          {#if app.cloudLoading}
            <Icon
              icon="ph:circle-notch"
              class="animate-spin text-base text-[var(--color-accent)]"
            />
          {/if}
        </div>

        {#if cloudTrail.length > 0}
          <div class="mb-3 flex flex-wrap items-center gap-1 text-xs">
            <button
              class="rounded px-2 py-1 text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface)] hover:text-[var(--color-ink)]"
              onclick={() => openCloudFolder(null)}
            >
              My Drive
            </button>

            {#each cloudTrail as folder, index}
              <Icon
                icon="ph:caret-right"
                class="text-[10px] text-[var(--color-ink-muted)]"
              />
              <button
                class="rounded px-2 py-1 transition hover:bg-[var(--color-surface)]
                  {index === cloudTrail.length - 1
                  ? 'font-medium'
                  : 'text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
                onclick={() => openCloudFolder(folder.id)}
              >
                {folder.name}
              </button>
            {/each}
          </div>
        {/if}

        {#if app.cloudFolders.length > 0}
          <div
            class="mb-4 grid grid-cols-[repeat(auto-fill,minmax(210px,1fr))] gap-2"
          >
            {#each app.cloudFolders as folder (folder.id)}
              <button
                class="flex items-center gap-2.5 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-sunken)] px-3 py-2.5 text-left transition hover:border-[var(--color-accent)] hover:bg-[var(--color-surface)]"
                onclick={() => openCloudFolder(folder.id)}
              >
                <Icon
                  icon="ph:folder-fill"
                  class="shrink-0 text-2xl text-[var(--color-ink-muted)]"
                />
                <span class="truncate text-xs font-medium">{folder.name}</span>
              </button>
            {/each}
          </div>
        {/if}

        {#if app.cloudDocuments.length > 0}
          <h2
            class="mb-2 text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]"
          >
            Documents
          </h2>
          <div
            class="mb-4 grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-3"
          >
            {#each app.cloudDocuments as entry (entry.id)}
              {@const linked = linkedDocument(entry.id)}
              {@render cloudCard(
                "ph:file-text",
                entry.title,
                linked
                  ? `Document · ${entry.role}`
                  : `${entry.role} · ${formatDate(entry.updated_at)}`,
                linked,
                linked ? () => openTarget(linked.path) : null,
                () => ondownloaddocument(entry.id, entry.title),
                linked ? () => onremovedownload(linked.path) : null,
              )}
            {/each}
          </div>
        {/if}

        {#if app.cloudFiles.length > 0}
          <h2
            class="mb-2 text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]"
          >
            Images and fonts
          </h2>
          <div
            class="mb-4 grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-3"
          >
            {#each app.cloudFiles as file (file.id)}
              <div
                class="flex items-center gap-2.5 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-3 transition hover:border-[var(--color-accent)]"
              >
                <Icon
                  icon={api.isImagePath(file.name)
                    ? "ph:image"
                    : /\.(ttf|otf|ttc|otc)$/i.test(file.name)
                      ? "ph:text-aa"
                      : "ph:file"}
                  class="shrink-0 text-xl text-[var(--color-accent)]"
                />
                <div class="min-w-0 flex-1">
                  <p class="truncate text-xs font-medium" title={file.name}>
                    {file.name}
                  </p>
                  <p class="truncate text-[10px] text-[var(--color-ink-muted)]">
                    {formatDate(file.created_at)}
                  </p>
                </div>
                <button
                  class="shrink-0 rounded-md border border-[var(--color-line)] p-1.5 text-[var(--color-ink-muted)] transition hover:border-[var(--color-accent)] hover:text-[var(--color-accent)]"
                  onclick={() => ondownloadfile(file.id, file.name)}
                  title="Add to shared assets on this device"
                  aria-label="Download"
                >
                  <Icon icon="ph:download-simple" />
                </button>
              </div>
            {/each}
          </div>
        {/if}

        {#if app.spaces.length === 0 && app.cloudDocuments.length === 0 && app.cloudFolders.length === 0 && app.cloudFiles.length === 0}
          <div
            class="flex flex-col items-center justify-center gap-3 py-16 text-[var(--color-ink-muted)]"
          >
            <Icon icon="ph:cloud" class="text-5xl" />
            <p class="text-sm">Nothing here yet.</p>
          </div>
        {/if}

        {#if app.spaces.length > 0}
          <h2
            class="mb-2 text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]"
          >
            Spaces
          </h2>
        {/if}
        <div class="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-3">
          {#each app.spaces as space (space.id)}
            {@const linked = linkedSpace(space.id)}
            {@render cloudCard(
              "ph:folder-star",
              space.name,
              linked
                ? `Project · ${space.role}`
                : `${space.role} · ${formatDate(space.updated_at)}`,
              linked,
              linked ? () => openTarget(linked.path) : null,
              () => onclonespace(space.id, space.name),
              space.role === "owner" ? () => ondeletespace(space.id) : null,
            )}
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>
