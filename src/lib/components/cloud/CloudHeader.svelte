<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { CloudFolder } from "$lib/ts/api";

  interface Props {
    title: string;
    folderPath: CloudFolder[];
    rootLabel: string;
    rootIcon: string;
    showActions: boolean;
    loading: boolean;
    dropTarget: string | null;
    searchQuery?: string;
    onnavigate: (folderId: string | null) => void;
    ondragovercrumb: (event: DragEvent, folderId: string) => void;
    ondragleavecrumb: (folderId: string) => void;
    ondropcrumb: (event: DragEvent, folderId: string) => void;
    onnewdocument: () => void;
    onnewproject: () => void;
    onnewfolder: () => void;
    onupload: () => void;
  }

  let {
    title,
    folderPath,
    rootLabel,
    rootIcon,
    showActions,
    loading,
    dropTarget,
    searchQuery = $bindable(""),
    onnavigate,
    ondragovercrumb,
    ondragleavecrumb,
    ondropcrumb,
    onnewdocument,
    onnewproject,
    onnewfolder,
    onupload,
  }: Props = $props();

  let newMenuOpen = $state(false);

  const secondaryNewItems = [
    { label: "Project", description: "Multi-file workspace", icon: "mdi:folder-star", color: "text-indigo-500", action: () => onnewproject() },
    { label: "Folder", description: "Organize your files", icon: "mdi:folder-plus", color: "text-yellow-500", action: () => onnewfolder() },
  ];

  function runNewItem(action: () => void) {
    newMenuOpen = false;
    action();
  }

  function handleWindowClick(event: MouseEvent) {
    if (!(event.target as HTMLElement).closest("[data-new-menu]")) {
      newMenuOpen = false;
    }
  }

  function crumbClass(folderId: string) {
    return dropTarget === folderId
      ? "bg-[var(--color-accent-soft)] text-[var(--color-accent)]"
      : "";
  }
</script>

<svelte:window onclick={handleWindowClick} />

<header class="mb-6">
  <nav class="mb-2 flex flex-wrap items-center gap-1 text-xs text-[var(--color-ink-muted)]" aria-label="Folder path">
    <button
      class="flex items-center gap-1 rounded px-1.5 py-0.5 transition-colors hover:text-[var(--color-ink)] {crumbClass('')}"
      onclick={() => onnavigate(null)}
      ondragover={(event) => ondragovercrumb(event, "")}
      ondragleave={() => ondragleavecrumb("")}
      ondrop={(event) => ondropcrumb(event, "")}
    >
      <Icon icon={rootIcon} class="text-sm" />
      {rootLabel}
    </button>
    {#each folderPath as folder (folder.id)}
      <span class="opacity-50">/</span>
      <button
        class="max-w-48 truncate rounded px-1.5 py-0.5 transition-colors hover:text-[var(--color-ink)] {crumbClass(folder.id)}"
        onclick={() => onnavigate(folder.id)}
        ondragover={(event) => ondragovercrumb(event, folder.id)}
        ondragleave={() => ondragleavecrumb(folder.id)}
        ondrop={(event) => ondropcrumb(event, folder.id)}
      >
        {folder.name}
      </button>
    {/each}
  </nav>

  <div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
    <div class="flex min-w-0 items-center gap-3">
      <h1 class="min-w-0 truncate text-2xl font-bold text-[var(--color-ink)]">{title}</h1>
      {#if loading}
        <Icon icon="mdi:loading" class="shrink-0 animate-spin text-xl text-[var(--color-accent)]" />
      {/if}
    </div>

    <div class="flex flex-wrap items-center gap-2">
      <div class="relative">
        <Icon icon="mdi:magnify" class="absolute left-3 top-1/2 -translate-y-1/2 text-lg text-[var(--color-ink-muted)]" />
        <input
          type="search"
          bind:value={searchQuery}
          placeholder="Filter this view"
          aria-label="Filter items"
          class="w-56 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] py-2 pl-9 pr-3 text-sm text-[var(--color-ink)] transition-colors focus:border-[var(--color-accent)] focus:outline-none"
        />
      </div>

      {#if showActions}
        <button
          class="flex items-center gap-1.5 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm font-medium text-[var(--color-ink)] transition-colors hover:bg-[var(--color-surface-sunken)]"
          onclick={onupload}
        >
          <Icon icon="mdi:tray-arrow-up" class="text-lg" />
          Upload
        </button>

        <div class="relative flex" data-new-menu>
          <button
            class="flex items-center gap-1.5 rounded-l-lg bg-[var(--color-accent)] py-2 pl-3 pr-2.5 text-sm font-semibold text-white transition hover:opacity-90"
            onclick={onnewdocument}
          >
            <Icon icon="mdi:file-document-plus-outline" class="text-lg" />
            New document
          </button>
          <button
            class="rounded-r-lg border-l border-white/25 bg-[var(--color-accent)] px-2 py-2 text-white transition hover:opacity-90"
            aria-label="More create options"
            aria-haspopup="menu"
            aria-expanded={newMenuOpen}
            onclick={() => (newMenuOpen = !newMenuOpen)}
          >
            <Icon icon="mdi:chevron-down" class="text-lg" />
          </button>

          {#if newMenuOpen}
            <div role="menu" class="absolute right-0 top-full z-30 mt-1 w-60 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-1 shadow-xl">
              {#each secondaryNewItems as item (item.label)}
                <button
                  role="menuitem"
                  class="flex w-full items-center gap-3 rounded-md px-3 py-2 text-left hover:bg-[var(--color-surface-sunken)]"
                  onclick={() => runNewItem(item.action)}
                >
                  <Icon icon={item.icon} class="text-xl {item.color}" />
                  <span>
                    <span class="block text-sm font-medium text-[var(--color-ink)]">{item.label}</span>
                    <span class="block text-xs text-[var(--color-ink-muted)]">{item.description}</span>
                  </span>
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</header>
