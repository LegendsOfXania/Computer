<script lang="ts">
  import "../app.css";
  import Sidebar from "$lib/components/Sidebar.svelte";
  let { children } = $props();
  let hover = $state(false);
</script>

<Sidebar bind:hover />
<main class:expanded={hover}>{@render children()}</main>

<style>
  main {
    min-height: 100vh;
    padding-left: var(--sidebar-width-collapsed);
    transition: padding-left var(--sidebar-transition-duration) ease;
  }
  main.expanded {
    padding-left: var(--sidebar-width-expanded);
  }
</style>
