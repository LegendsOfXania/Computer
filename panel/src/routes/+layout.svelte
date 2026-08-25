<script lang="ts">
  import favicon from "$lib/assets/favicon.svg";
  import "../app.css";
  import Sidebar from "$lib/components/Sidebar.svelte";

  let { children } = $props();
  let sidebarHover = $state(false);
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
</svelte:head>

<Sidebar bind:hover={sidebarHover} />

<main class="main-content" class:sidebar-expanded={sidebarHover}>
  {@render children()}
</main>

<style>
  .main-content {
    min-height: 100vh;
    padding-left: var(--sidebar-width-collapsed);
    transition: padding-left var(--sidebar-transition-duration) ease;
  }

  .main-content.sidebar-expanded {
    padding-left: var(--sidebar-width-expanded);
  }
</style>
