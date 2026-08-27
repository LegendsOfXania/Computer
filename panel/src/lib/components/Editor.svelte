<script lang="ts">
  import { SvelteFlow, Background, type Node, type Edge } from "@xyflow/svelte";
  import "@xyflow/svelte/dist/base.css";
  import {
    layoutSequenceEntries,
    layoutStaticEntries,
  } from "$lib/editor/layout";
  import { appStore } from "$lib/stores/app.svelte";

  let nodes = $state.raw<Node[]>([]);
  let edges = $state.raw<Edge[]>([]);

  $effect(() => {
    const page = appStore.selectedPage;
    const entries = appStore.entries;

    if (page === null) {
      nodes = [];
      edges = [];
      return;
    }

    if (page.page_type !== "sequence") {
      const layout = layoutStaticEntries(entries);
      nodes = layout.nodes;
      edges = layout.edges;
      return;
    }

    const layout = layoutSequenceEntries(page.id, entries);
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
  >
    <Background gap={20} />
  </SvelteFlow>
</div>

<style>
  .editor {
    flex: 1;
    min-width: 0;
    height: 100%;

    --xy-background-color: var(--surface);
    --xy-background-pattern-dots-color: var(--border-muted);

    --xy-edge-stroke: var(--accent);
    --xy-edge-stroke-width: 2px;
    --xy-edge-stroke-selected: var(--accent);
    --xy-connectionline-stroke: var(--accent);
    --xy-connectionline-stroke-width: 2px;

    --xy-handle-background-color: var(--on-accent);
    --xy-handle-border-color: var(--accent);
    --xy-handle-width: 10px;
    --xy-handle-height: 10px;
  }

  :global(.editor .svelte-flow__node) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 10px 18px;
    font-size: 13px;
    font-weight: 800;
    border: none;
    border-radius: var(--radius);
    color: var(--on-accent);
    background-color: var(--accent) !important;
    box-shadow: 3px 3px 0 var(--accent-shadow) !important;
    cursor: pointer;
    transition:
      box-shadow 0.15s ease,
      background-color 0.15s ease;
  }

  :global(.editor .svelte-flow__node:hover) {
    background-color: var(--accent-hover) !important;
    box-shadow: 4px 4px 0 var(--accent-shadow) !important;
  }

  :global(.editor .svelte-flow__node.selected),
  :global(.editor .svelte-flow__node.selected:hover) {
    background-color: var(--accent-hover) !important;
    box-shadow: 0 0 0 transparent !important;
  }

  :global(.editor .svelte-flow__node.dragging) {
    transition: none !important;
  }
  :global(.editor .svelte-flow__handle:not(.connectable)) {
    display: none;
  }
</style>
