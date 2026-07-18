<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { bootstrap } from "$lib/ts/state.svelte";

  let { children } = $props();

  onMount(bootstrap);
</script>

{@render children()}
