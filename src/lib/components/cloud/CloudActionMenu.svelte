<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { CloudAction } from "#lib/ts/cloud-entries";
  import FolderColorSwatches from "../FolderColorSwatches.svelte";
  import { clampMenu } from "#lib/ts/menu-position";

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
  class="menu fixed z-[100] w-52"
  style="left: {x}px; top: {y}px"
  role="menu"
>
  {#if onsetcolor}
    <FolderColorSwatches {color} onchoose={chooseColor} />
  {/if}
  {#each regularActions as action (action.label)}
    <button
      role="menuitem"
      class="menu-item"
      onclick={(event) => {
        event.stopPropagation();
        run(action);
      }}
    >
      <Icon icon={action.icon} class="text-base {action.color ?? 'text-[var(--color-ink-muted)]'}" />
      {action.label}
    </button>
  {/each}
  {#if dangerActions.length > 0}
    <div class="menu-separator"></div>
    {#each dangerActions as action (action.label)}
      <button
        role="menuitem"
        class="menu-item menu-item-danger"
        onclick={(event) => {
          event.stopPropagation();
          run(action);
        }}
      >
        <Icon icon={action.icon} class="text-base" />
        {action.label}
      </button>
    {/each}
  {/if}
</div>
