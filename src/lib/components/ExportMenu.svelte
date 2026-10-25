<script lang="ts">
  import Icon from "@iconify/svelte";
  import * as api from "#lib/ts/api";
  import { exportFormats } from "#lib/ts/export-formats";

  interface Props {
    onexport: (formatId: string) => void;
  }

  let { onexport }: Props = $props();

  let open = $state(false);
  let pandocVersion = $state<string | null>(null);
  let pandocChecked = $state(false);

  const typstFormats = exportFormats.filter((format) => !format.pandoc);
  const pandocFormats = exportFormats.filter((format) => format.pandoc);

  async function toggle(event: MouseEvent) {
    event.stopPropagation();
    open = !open;
    if (open && !pandocChecked) {
      pandocVersion = await api.pandocVersion().catch(() => null);
      pandocChecked = true;
    }
  }

  function choose(formatId: string) {
    open = false;
    onexport(formatId);
  }
</script>

<svelte:window onclick={() => (open = false)} />

<div class="relative">
  <button class="btn btn-ghost {open ? 'bg-[var(--color-surface-sunken)] text-[var(--color-ink)]' : ''}" aria-haspopup="menu" aria-expanded={open} onclick={toggle}>
    <Icon icon="ph:export" />
    Export
  </button>

  {#if open}
    <div role="menu" class="menu absolute right-0 top-full z-30 mt-1 w-56">
      <p class="section-label px-2.5 pb-1 pt-1.5">Typst</p>
      {#each typstFormats as format (format.id)}
        <button role="menuitem" class="menu-item" onclick={() => choose(format.id)}>
          <Icon icon={format.icon} class="text-sm text-[var(--color-ink-muted)]" />
          <span class="flex-1">{format.label}</span>
          <span class="text-[10px] text-[var(--color-ink-subtle)]">.{format.extension}</span>
        </button>
      {/each}

      <div class="menu-separator"></div>
      <p class="section-label flex items-center gap-1.5 px-2.5 pb-1 pt-1.5">
        Pandoc
        {#if pandocChecked && !pandocVersion}
          <span class="font-normal normal-case text-[var(--color-danger)]">unavailable</span>
        {/if}
      </p>
      {#each pandocFormats as format (format.id)}
        <button
          role="menuitem"
          class="menu-item disabled:pointer-events-none disabled:opacity-50"
          disabled={pandocChecked && !pandocVersion}
          onclick={() => choose(format.id)}
        >
          <Icon icon={format.icon} class="text-sm text-[var(--color-ink-muted)]" />
          <span class="flex-1">{format.label}</span>
          <span class="text-[10px] text-[var(--color-ink-subtle)]">.{format.extension}</span>
        </button>
      {/each}
      {#if pandocChecked && !pandocVersion}
        <p class="px-2.5 pb-1.5 pt-1 text-[11px] leading-snug text-[var(--color-ink-muted)]">
          Pandoc could not be started. Reinstall Typst Desktop to restore it.
        </p>
      {/if}
    </div>
  {/if}
</div>
