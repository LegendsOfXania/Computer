<script lang="ts">
  import Header from "$lib/components/Header.svelte";
  import Editor from "$lib/components/Editor.svelte";
  import Inspector from "$lib/components/Inspector.svelte";
</script>

<svelte:head>
  <title>Computer</title>
</svelte:head>

<div class="panel">
  <Header />

  <div class="workspace">
    <Editor />
    <Inspector />
  </div>
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100vh;
  }

  .workspace {
    display: flex;
    flex: 1;
    min-height: 0;
  }
</style>
