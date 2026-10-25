<script lang="ts">
  import { avatarFor } from "#lib/ts/review/client";
  import type { PersonSummary } from "#lib/ts/review/people";

  let { person, sizeClass = "h-6 w-6", textClass = "text-[10px]" }: { person: PersonSummary; sizeClass?: string; textClass?: string } = $props();

  let imageSource = $state<string | null>(null);

  $effect(() => {
    const personId = person.id;
    const hasAvatar = !!person.avatar_url;
    imageSource = null;
    if (!hasAvatar) return;
    avatarFor(personId).then((source) => {
      if (person.id === personId) imageSource = source;
    });
  });
</script>

{#if imageSource}
  <img src={imageSource} alt={person.name} title={person.name} class="{sizeClass} rounded-full object-cover ring-2 ring-[var(--color-surface)]" />
{:else}
  <span title={person.name} class="{sizeClass} {textClass} flex items-center justify-center rounded-full bg-[var(--color-accent-soft)] font-bold text-[var(--color-accent)] ring-2 ring-[var(--color-surface)]">
    {person.name.charAt(0).toUpperCase()}
  </span>
{/if}
