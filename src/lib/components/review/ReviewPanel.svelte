<script lang="ts">
  import { reviewSessionStore } from "$lib/ts/review/session";
  import { reviewPanelOpenStore } from "$lib/ts/review/client";
  import CommentsPanel from "./CommentsPanel.svelte";
</script>

{#if $reviewSessionStore && $reviewPanelOpenStore}
  {#key $reviewSessionStore}
    <CommentsPanel
      docId={$reviewSessionStore.documentId}
      canComment={$reviewSessionStore.role !== "viewer"}
      canReview={$reviewSessionStore.role === "owner" || $reviewSessionStore.role === "editor"}
      isOwner={$reviewSessionStore.role === "owner"}
      onClose={() => reviewPanelOpenStore.set(false)}
    />
  {/key}
{/if}
