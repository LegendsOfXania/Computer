<script lang="ts">
  import { SvelteFlow, Background, type Node, type Edge } from "@xyflow/svelte";
  import "@xyflow/svelte/dist/base.css";
  import { layoutSequenceEntries } from "$lib/editor/layout";
  import { mockPageContent } from "$lib/mocks/page-content";
  import { appStore } from "$lib/stores/app.svelte";

  let nodes = $state.raw<Node[]>([]);
  let edges = $state.raw<Edge[]>([]);

  $effect(() => {
    const page = appStore.selectedPage;

    if (page === null) {
      nodes = [];
      edges = [];
      return;
    }

    const content = mockPageContent(page);

    if (content.page.page_type !== "sequence") {
      nodes = content.entries.map((entry, index) => ({
        id: entry.id,
        type: "default",
        position: { x: (index % 4) * 260, y: Math.floor(index / 4) * 100 },
        data: { label: entry.id },
      }));
      edges = [];
      return;
    }

    const layout = layoutSequenceEntries(content.page.id, content.entries);
    nodes = layout.nodes;
    edges = layout.edges;
  });
</script>

<div class="editor">
  <SvelteFlow bind:nodes bind:edges fitView>
    <Background gap={20} />
  </SvelteFlow>
</div>

<style>
  .editor {
    width: 100%;
    height: 100%;
    min-height: calc(100vh - var(--header-height));

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
</style>
