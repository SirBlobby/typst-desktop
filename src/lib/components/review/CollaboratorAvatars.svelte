<script lang="ts">
  import { onMount } from "svelte";
  import type { Awareness } from "y-protocols/awareness";
  import { avatarFor } from "$lib/ts/review/client";

  interface Collaborator {
    clientId: number;
    id: string | null;
    name: string;
    color: string;
    isLocal: boolean;
  }

  let { awareness }: { awareness: Awareness } = $props();

  let collaborators = $state<Collaborator[]>([]);
  let avatars = $state<Record<string, string | null>>({});

  function readCollaborators() {
    const list: Collaborator[] = [];
    awareness.getStates().forEach((state, clientId) => {
      const user = state.user;
      if (!user) return;
      list.push({
        clientId,
        id: user.id ?? null,
        name: user.name ?? "Anonymous",
        color: user.color ?? "#888888",
        isLocal: clientId === awareness.clientID,
      });
    });
    collaborators = list;
    for (const collaborator of list) {
      const userId = collaborator.id;
      if (!userId || userId in avatars) continue;
      avatars[userId] = null;
      avatarFor(userId).then((source) => (avatars[userId] = source));
    }
  }

  onMount(() => {
    readCollaborators();
    awareness.on("change", readCollaborators);
    return () => awareness.off("change", readCollaborators);
  });
</script>

{#if collaborators.length > 0}
  <div class="flex items-center -space-x-1.5">
    {#each collaborators as collaborator (collaborator.clientId)}
      {@const source = collaborator.id ? avatars[collaborator.id] : null}
      <div
        class="flex h-7 w-7 items-center justify-center overflow-hidden rounded-full border-2 text-[10px] font-bold text-white"
        style="background-color: {collaborator.color}; border-color: {collaborator.color}; z-index: {collaborator.isLocal ? 10 : 1};"
        title={collaborator.name + (collaborator.isLocal ? " (You)" : "")}
      >
        {#if source}
          <img src={source} alt={collaborator.name} class="h-full w-full object-cover" />
        {:else}
          {collaborator.name.substring(0, 2).toUpperCase()}
        {/if}
      </div>
    {/each}
  </div>
{/if}
