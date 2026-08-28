<script lang="ts">
  import { SvelteFlow, Background, type Node, type Edge } from "@xyflow/svelte";
  import "@xyflow/svelte/dist/base.css";
  import {
    layoutSequenceEntries,
    layoutStaticEntries,
  } from "$lib/editor/layout";
  import { appStore } from "$lib/stores/app.svelte";
  let nodes = $state.raw<Node[]>([]),
    edges = $state.raw<Edge[]>([]);
  $effect(() => {
    const page = appStore.selectedPage;
    if (!page) {
      nodes = [];
      edges = [];
      return;
    }
    const layout =
      page.page_type === "sequence"
        ? layoutSequenceEntries(page.id, appStore.entries)
        : layoutStaticEntries(appStore.entries);
    nodes = layout.nodes;
    edges = layout.edges;
  });
</script>

<div class="editor">
  <SvelteFlow
    bind:nodes
    bind:edges
    fitView
    onnodeclick={({ node }) => appStore.selectEntry(node.id)}
    onpaneclick={() => appStore.clearEntrySelection()}
    ><Background gap={20} /></SvelteFlow
  >
</div>

<style>
  .editor {
    flex: 1;
    min-width: 0;
    height: 100%;
    --xy-background-color: #131313;
    --xy-background-pattern-dots-color: var(--border-muted);
    --xy-edge-stroke: var(--accent);
    --xy-edge-stroke-width: 2px;
  }
  :global(.editor .svelte-flow__node) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 10px 18px;
    font-size: 13px;
    font-weight: 800;
    border: 0;
    border-radius: var(--radius);
    color: var(--on-accent);
    background: var(--accent) !important;
    box-shadow: 3px 3px 0 var(--accent-shadow) !important;
    cursor: pointer;
  }
  :global(.editor .svelte-flow__node:hover) {
    background: var(--accent-hover) !important;
    box-shadow: 4px 4px 0 var(--accent-shadow) !important;
  }
  :global(.editor .svelte-flow__node.selected) {
    background: var(--accent-hover) !important;
    box-shadow: none !important;
  }
  :global(.editor .svelte-flow__handle:not(.connectable)) {
    display: none;
  }
</style>
