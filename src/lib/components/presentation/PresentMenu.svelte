<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { PresentationLayout } from "#lib/ts/presentation-sync";

  let { disabled = false, onpresent }: { disabled?: boolean; onpresent: (layout: PresentationLayout) => void } =
    $props();

  let open = $state(false);

  const options: { layout: PresentationLayout; label: string; description: string; icon: string }[] = [
    { layout: "audience", label: "Present", description: "Full screen in this window", icon: "ph:presentation" },
    { layout: "speaker", label: "Speaker view", description: "Slides in a second window", icon: "ph:user-sound" },
  ];

  function toggle(event: MouseEvent) {
    event.stopPropagation();
    open = !open;
  }

  function choose(layout: PresentationLayout) {
    open = false;
    onpresent(layout);
  }
</script>

<svelte:window onclick={() => (open = false)} />

<div class="relative">
  <button
    class="btn btn-ghost {open ? 'bg-[var(--color-surface-sunken)] text-[var(--color-ink)]' : ''}"
    title={disabled ? "Compile the document to present it" : "Present"}
    aria-haspopup="menu"
    aria-expanded={open}
    {disabled}
    onclick={toggle}
  >
    <Icon icon="ph:presentation" />
    Present
    <Icon icon="ph:caret-down" class="text-xs" />
  </button>

  {#if open}
    <div role="menu" class="menu absolute right-0 top-full z-30 mt-1 w-60">
      {#each options as option (option.layout)}
        <button role="menuitem" class="menu-item" onclick={() => choose(option.layout)}>
          <Icon icon={option.icon} class="text-sm text-[var(--color-ink-muted)]" />
          <span class="flex flex-1 flex-col">
            <span>{option.label}</span>
            <span class="text-xs text-[var(--color-ink-muted)]">{option.description}</span>
          </span>
        </button>
      {/each}
    </div>
  {/if}
</div>
