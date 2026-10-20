<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { CloudAction } from "$lib/ts/cloud-entries";
  import { defaultFolderColor, folderColorOptions } from "$lib/ts/folder-colors";
  import { clampMenu } from "$lib/ts/menu-position";

  interface Props {
    actions: CloudAction[];
    x: number;
    y: number;
    color?: string | null;
    onsetcolor?: ((color: string | null) => void) | null;
    onclose: () => void;
  }

  let { actions, x, y, color = null, onsetcolor = null, onclose }: Props = $props();

  function chooseColor(value: string | null) {
    const setColor = onsetcolor;
    const currentColor = color;
    onclose();
    if (setColor && value !== currentColor) setColor(value);
  }

  const regularActions = $derived(actions.filter((action) => !action.danger));
  const dangerActions = $derived(actions.filter((action) => action.danger));

  function run(action: CloudAction) {
    onclose();
    action.run();
  }
</script>

<svelte:window onclick={onclose} onkeydown={(event) => event.key === "Escape" && onclose()} />

<div
  use:clampMenu
  class="fixed z-[100] w-52 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] py-1 shadow-xl"
  style="left: {x}px; top: {y}px"
  role="menu"
>
  {#if onsetcolor}
    <div class="grid grid-cols-5 gap-1.5 border-b border-[var(--color-line)] px-3 pb-2.5 pt-1.5">
      {#each folderColorOptions as option (option.label)}
        {@const selected = color === option.value}
        <button
          role="menuitemradio"
          aria-checked={selected}
          aria-label="{option.label} folder color"
          title={option.label}
          class="flex h-6 w-6 items-center justify-center rounded-full transition-transform hover:scale-110 {selected
            ? 'ring-2 ring-[var(--color-ink)] ring-offset-2 ring-offset-[var(--color-surface)]'
            : ''}"
          style="background-color: {option.value ?? defaultFolderColor};"
          onclick={(event) => {
            event.stopPropagation();
            chooseColor(option.value);
          }}
        >
          {#if selected}
            <Icon icon="mdi:check" class="text-sm text-white" />
          {/if}
        </button>
      {/each}
    </div>
  {/if}
  {#each regularActions as action (action.label)}
    <button
      role="menuitem"
      class="flex w-full items-center gap-2 px-4 py-2 text-left text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]"
      onclick={(event) => {
        event.stopPropagation();
        run(action);
      }}
    >
      <Icon icon={action.icon} class="text-lg {action.color ?? ''}" />
      {action.label}
    </button>
  {/each}
  {#if dangerActions.length > 0}
    <div class="my-1 h-px bg-[var(--color-line)]"></div>
    {#each dangerActions as action (action.label)}
      <button
        role="menuitem"
        class="flex w-full items-center gap-2 px-4 py-2 text-left text-sm text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10"
        onclick={(event) => {
          event.stopPropagation();
          run(action);
        }}
      >
        <Icon icon={action.icon} class="text-lg" />
        {action.label}
      </button>
    {/each}
  {/if}
</div>
