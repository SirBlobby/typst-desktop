<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { CloudFolder } from "#lib/ts/api";

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
    { label: "Project", description: "Multi-file workspace", icon: "ph:folder-star", action: () => onnewproject() },
    { label: "Folder", description: "Organize your files", icon: "ph:folder-plus", action: () => onnewfolder() },
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

{#snippet crumb(folderId: string, label: string, icon?: string)}
  <button
    class="flex max-w-48 items-center gap-1 truncate rounded-md px-1.5 py-0.5 transition-colors hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] {crumbClass(folderId)}"
    onclick={() => onnavigate(folderId || null)}
    ondragover={(event) => ondragovercrumb(event, folderId)}
    ondragleave={() => ondragleavecrumb(folderId)}
    ondrop={(event) => ondropcrumb(event, folderId)}
  >
    {#if icon}<Icon {icon} class="shrink-0" />{/if}
    {label}
  </button>
{/snippet}

<header class="mb-5">
  {#if folderPath.length > 0}
    <nav class="mb-1.5 flex flex-wrap items-center gap-0.5 text-xs text-[var(--color-ink-muted)]" aria-label="Folder path">
      {@render crumb("", rootLabel, rootIcon)}
      {#each folderPath.slice(0, -1) as folder (folder.id)}
        <Icon icon="ph:caret-right" class="text-[10px] text-[var(--color-ink-subtle)]" />
        {@render crumb(folder.id, folder.name)}
      {/each}
    </nav>
  {/if}

  <div class="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
    <div class="flex min-w-0 items-center gap-2.5">
      <h1 class="min-w-0 truncate text-xl font-semibold tracking-tight">{title}</h1>
      {#if loading}
        <Icon icon="ph:circle-notch" class="shrink-0 animate-spin text-base text-[var(--color-accent)]" />
      {/if}
    </div>

    <div class="flex flex-wrap items-center gap-2">
      <div class="relative">
        <Icon icon="ph:magnifying-glass" class="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-sm text-[var(--color-ink-subtle)]" />
        <input
          type="search"
          bind:value={searchQuery}
          placeholder="Filter this view"
          aria-label="Filter items"
          class="field field-sm h-7 w-56 pl-8"
        />
      </div>

      {#if showActions}
        <button class="btn btn-secondary" onclick={onupload}>
          <Icon icon="ph:upload-simple" />
          Upload
        </button>

        <div class="relative flex" data-new-menu>
          <button class="btn btn-primary rounded-r-none pr-2" onclick={onnewdocument}>
            <Icon icon="ph:file-plus" />
            New document
          </button>
          <button
            class="btn btn-primary rounded-l-none border-l border-l-black/10 px-1.5"
            aria-label="More create options"
            aria-haspopup="menu"
            aria-expanded={newMenuOpen}
            onclick={() => (newMenuOpen = !newMenuOpen)}
          >
            <Icon icon="ph:caret-down" />
          </button>

          {#if newMenuOpen}
            <div role="menu" class="menu absolute right-0 top-full z-30 mt-1.5 w-56">
              {#each secondaryNewItems as item (item.label)}
                <button
                  role="menuitem"
                  class="menu-item gap-2.5 py-2"
                  onclick={() => runNewItem(item.action)}
                >
                  <Icon icon={item.icon} class="text-base text-[var(--color-accent)]" />
                  <span>
                    <span class="block font-medium">{item.label}</span>
                    <span class="block text-[11px] text-[var(--color-ink-muted)]">{item.description}</span>
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
