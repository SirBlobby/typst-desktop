<script lang="ts">
  import { avatarFor } from "$lib/ts/review/client";
  import { app } from "$lib/ts/state.svelte";

  let { sizeClass = "h-7 w-7 text-xs" }: { sizeClass?: string } = $props();

  let avatarSource = $state<string | null>(null);

  const userId = $derived(app.account?.user_id ?? null);
  const initial = $derived(app.account?.username?.charAt(0).toUpperCase() ?? "?");

  $effect(() => {
    const currentUserId = userId;
    avatarSource = null;
    if (!currentUserId) return;
    avatarFor(currentUserId).then((source) => {
      if (userId === currentUserId) avatarSource = source;
    });
  });
</script>

<span
  class="flex shrink-0 items-center justify-center overflow-hidden rounded-full bg-[var(--color-accent-soft)] font-bold text-[var(--color-accent)] {sizeClass}"
>
  {#if avatarSource}
    <img src={avatarSource} alt={app.account?.username ?? "Account"} class="h-full w-full object-cover" />
  {:else}
    {initial}
  {/if}
</span>
