<script lang="ts">
  import Header from "./lib/components/Header.svelte";
  import PageSidebar from "./lib/components/PageSidebar.svelte";
  import PageEditor from "./lib/components/PageEditor.svelte";
  import Inspector from "./lib/components/Inspector.svelte";
  import { workspace } from "./lib/state/workspace.svelte";
</script>

<div class="app">
  <Header />
  <main>
    <PageSidebar /><PageEditor page={workspace.selectedPage} /><Inspector />
  </main>
</div>

<style>
  .app {
    width: 100%;
    height: 100%;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }
  main {
    min-height: 0;
    display: flex;
    flex: 1;
  }
  @media (max-width: 980px) {
    main {
      min-width: 900px;
    }
  }
</style>
