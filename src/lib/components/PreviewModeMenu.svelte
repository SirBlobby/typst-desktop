<script lang="ts">
  import Icon from "@iconify/svelte";
  import { previewMode, previewModeOptions, setPreviewMode, type PreviewMode } from "$lib/ts/preview-mode.svelte";

  let open = $state(false);

  const currentOption = $derived(
    previewModeOptions.find((option) => option.value === previewMode.current) ?? previewModeOptions[0],
  );

  function toggle(event: MouseEvent) {
    event.stopPropagation();
    open = !open;
  }

  function choose(mode: PreviewMode) {
    open = false;
    setPreviewMode(mode);
  }
</script>

<svelte:window onclick={() => (open = false)} />

<div class="relative">
  <button
    class="btn btn-ghost {open ? 'bg-[var(--color-surface-sunken)] text-[var(--color-ink)]' : ''}"
    title="View mode"
    aria-haspopup="menu"
    aria-expanded={open}
    onclick={toggle}
  >
    <Icon icon={currentOption.icon} />
    View
    <Icon icon="ph:caret-down" class="text-xs" />
  </button>

  {#if open}
    <div role="menu" class="menu absolute right-0 top-full z-30 mt-1 w-52">
      {#each previewModeOptions as option (option.value)}
        <button
          role="menuitemradio"
          aria-checked={previewMode.current === option.value}
          class="menu-item"
          onclick={() => choose(option.value)}
        >
          <Icon icon={option.icon} class="text-sm text-[var(--color-ink-muted)]" />
          <span class="flex-1">{option.label}</span>
          {#if previewMode.current === option.value}
            <Icon icon="ph:check" class="text-sm text-[var(--color-accent)]" />
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>
