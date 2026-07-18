<script lang="ts">
  import Icon from "@iconify/svelte";
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  const appWindow = getCurrentWindow();

  let maximized = $state(false);

  async function sync() {
    try {
      maximized = await appWindow.isMaximized();
    } catch {
      maximized = false;
    }
  }

  onMount(() => {
    sync();
    const pending = appWindow.onResized(sync);
    return () => {
      pending.then((unlisten) => unlisten());
    };
  });
</script>

<div class="flex items-center">
  <button
    class="flex h-11 w-11 items-center justify-center text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]"
    onclick={() => appWindow.minimize()}
    aria-label="Minimize"
  >
    <Icon icon="ph:minus" class="text-sm" />
  </button>

  <button
    class="flex h-11 w-11 items-center justify-center text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]"
    onclick={async () => {
      await appWindow.toggleMaximize();
      sync();
    }}
    aria-label={maximized ? "Restore" : "Maximize"}
  >
    <Icon icon={maximized ? "ph:corners-in" : "ph:square"} class="text-sm" />
  </button>

  <button
    class="flex h-11 w-11 items-center justify-center text-[var(--color-ink-muted)] transition hover:bg-[var(--color-danger)] hover:text-white"
    onclick={() => appWindow.close()}
    aria-label="Close"
  >
    <Icon icon="ph:x" class="text-sm" />
  </button>
</div>
