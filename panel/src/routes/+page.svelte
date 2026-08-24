<script lang="ts">
  import Header from "$lib/components/Header.svelte";
</script>

<svelte:head>
  <title>Computer</title>
</svelte:head>

<div class="panel">
  <Header />

  <main>
    <span>PANEL</span>
  </main>
</div>

<style>
  .panel {
    width: 100%;
    min-height: 100vh;
  }

  main {
    display: grid;
    min-height: calc(100vh - 64px);
    place-items: center;

    color: var(--text-muted);
    font-family: monospace;
    font-size: 14px;
    font-weight: 700;
    letter-spacing: 0.15em;
  }
</style>
