<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import { bootstrap } from "#lib/ts/state.svelte";

  let { children } = $props();

  onMount(() => {
    if (!page.url.pathname.startsWith("/present")) bootstrap();
  });
</script>

{@render children()}
