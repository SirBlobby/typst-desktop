<script lang="ts">
  import Icon from "@iconify/svelte";
  import { defaultFolderColor, folderColorOptions } from "#lib/ts/folder-colors";

  interface Props {
    color: string | null;
    onchoose: (color: string | null) => void;
  }

  let { color, onchoose }: Props = $props();
</script>

<div class="mb-1 grid grid-cols-5 gap-1.5 border-b border-[var(--color-line)] px-2 pb-2.5 pt-1.5">
  {#each folderColorOptions as option (option.label)}
    {@const selected = color === option.value}
    <button
      role="menuitemradio"
      aria-checked={selected}
      aria-label="{option.label} folder color"
      title={option.label}
      class="flex h-6 w-6 items-center justify-center rounded-full transition-transform hover:scale-110 {selected
        ? 'ring-2 ring-[var(--color-ink)] ring-offset-2 ring-offset-[var(--color-surface-raised)]'
        : ''}"
      style="background-color: {option.value ?? defaultFolderColor};"
      onclick={(event) => {
        event.stopPropagation();
        onchoose(option.value);
      }}
    >
      {#if selected}
        <Icon icon="ph:check" class="text-sm text-white" />
      {/if}
    </button>
  {/each}
</div>
