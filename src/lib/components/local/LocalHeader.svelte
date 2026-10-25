<script lang="ts">
  import Icon from "@iconify/svelte";
  import { browseTo } from "#lib/ts/state.svelte";

  interface Crumb {
    path: string;
    name: string;
  }

  interface Props {
    title: string;
    ancestors: Crumb[];
    showTrail: boolean;
    inShortcut: boolean;
    dropTarget: string | null;
    searchQuery: string;
    ondragovercrumb: (event: DragEvent, path: string) => void;
    ondragleavecrumb: (path: string) => void;
    ondropcrumb: (event: DragEvent, path: string) => void;
    onupload: () => void;
    onnewdocument: () => void;
    onnewproject: () => void;
    onnewfolder: () => void;
    onconvert: () => void;
  }

  let {
    title,
    ancestors,
    showTrail,
    inShortcut,
    dropTarget,
    searchQuery = $bindable(""),
    ondragovercrumb,
    ondragleavecrumb,
    ondropcrumb,
    onupload,
    onnewdocument,
    onnewproject,
    onnewfolder,
    onconvert,
  }: Props = $props();

  let newMenuOpen = $state(false);

  const createOptions = $derived([
    { label: "Project", description: "Multi-file workspace", icon: "ph:folder-star", action: onnewproject },
    { label: "Folder", description: "Organize your files", icon: "ph:folder-plus", action: onnewfolder },
    { label: "Convert to Typst", description: "From Word, LaTeX, Markdown or HTML", icon: "ph:file-arrow-up", action: onconvert },
  ]);
</script>

<svelte:window
  onclick={(event) => {
    if (!(event.target as HTMLElement).closest("[data-new-menu]")) newMenuOpen = false;
  }}
/>

{#snippet crumb(path: string, label: string, icon?: string)}
  <button
    class="flex max-w-48 items-center gap-1 truncate rounded-md px-1.5 py-0.5 transition hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]
      {dropTarget === path ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]' : ''}"
    onclick={() => browseTo(path)}
    ondragover={(event) => ondragovercrumb(event, path)}
    ondragleave={() => ondragleavecrumb(path)}
    ondrop={(event) => ondropcrumb(event, path)}
  >
    {#if icon}<Icon {icon} class="shrink-0" />{/if}
    {label}
  </button>
{/snippet}

<header class="shrink-0 px-6 pb-4 pt-5">
  {#if showTrail}
    <nav class="mb-1.5 flex flex-wrap items-center gap-0.5 text-xs text-[var(--color-ink-muted)]" aria-label="Location">
      {#if !inShortcut}
        {@render crumb("", "Workspace", "ph:hard-drives")}
      {/if}
      {#each ancestors as ancestor, index (ancestor.path)}
        {#if index > 0 || !inShortcut}
          <Icon icon="ph:caret-right" class="text-[10px] text-[var(--color-ink-subtle)]" />
        {/if}
        {@render crumb(ancestor.path, ancestor.name, index === 0 && inShortcut ? "ph:folder-simple" : undefined)}
      {/each}
    </nav>
  {/if}

  <div class="flex flex-wrap items-center gap-2">
    <h1 class="min-w-0 flex-1 truncate text-xl font-semibold tracking-tight">{title}</h1>

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

    <button class="btn btn-secondary" onclick={onupload}>
      <Icon icon="ph:upload-simple" />
      Import
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
          {#each createOptions as option (option.label)}
            <button
              role="menuitem"
              class="menu-item gap-2.5 py-2"
              onclick={() => {
                newMenuOpen = false;
                option.action();
              }}
            >
              <Icon icon={option.icon} class="text-base text-[var(--color-accent)]" />
              <span>
                <span class="block font-medium">{option.label}</span>
                <span class="block text-[11px] text-[var(--color-ink-muted)]">{option.description}</span>
              </span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</header>
