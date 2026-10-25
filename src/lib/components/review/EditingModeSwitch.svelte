<script lang="ts">
  import Icon from "@iconify/svelte";
  import { suggestingModeStore } from "#lib/ts/review/suggestions";

  let { canEdit }: { canEdit: boolean } = $props();

  const modes = [
    { suggesting: false, label: "Editing", icon: "ph:pencil-simple", description: "Edit the document directly" },
    { suggesting: true, label: "Suggesting", icon: "ph:note-pencil", description: "Edits become suggestions others can accept or reject" }
  ];

  let menuOpen = $state(false);
  let currentMode = $derived(modes.find((mode) => mode.suggesting === $suggestingModeStore) ?? modes[0]);
</script>

<svelte:window onclick={() => (menuOpen = false)} />

{#if canEdit}
  <div class="relative">
    <button
      onclick={(event) => { event.stopPropagation(); menuOpen = !menuOpen; }}
      class="btn {$suggestingModeStore ? 'bg-[var(--color-success)]/12 text-[var(--color-success)]' : 'btn-ghost'}"
      title="Change editing mode"
    >
      <Icon icon={currentMode.icon} class="text-sm" />
      {currentMode.label}
      <Icon icon="ph:caret-down" class="text-[10px] opacity-70" />
    </button>
    {#if menuOpen}
      <div class="menu absolute right-0 top-full z-[100] mt-1 w-64">
        {#each modes as mode (mode.label)}
          <button
            onclick={() => { suggestingModeStore.set(mode.suggesting); menuOpen = false; }}
            class="menu-item items-start gap-2.5 py-2"
          >
            <Icon icon={mode.icon} class="mt-0.5 text-sm text-[var(--color-ink-muted)]" />
            <span class="min-w-0 flex-1">
              <span class="block text-xs font-medium text-[var(--color-ink)]">{mode.label}</span>
              <span class="block text-[11px] text-[var(--color-ink-muted)]">{mode.description}</span>
            </span>
            {#if mode.suggesting === $suggestingModeStore}
              <Icon icon="ph:check" class="mt-0.5 text-sm text-[var(--color-accent)]" />
            {/if}
          </button>
        {/each}
      </div>
    {/if}
  </div>
{:else}
  <span class="chip h-7 rounded-md bg-[var(--color-success)]/12 text-xs text-[var(--color-success)]" title="You can suggest changes to this document">
    <Icon icon="ph:note-pencil" class="text-sm" />
    Suggesting
  </span>
{/if}
