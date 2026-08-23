<script lang="ts">
  import {
    Background,
    BackgroundVariant,
    Controls,
    MiniMap,
    SvelteFlow,
  } from "@xyflow/svelte";
  import "@xyflow/svelte/dist/style.css";
  import type { Edge, Node } from "@xyflow/svelte";
  import type { Page } from "../types";
  import EntryNode from "./EntryNode.svelte";
  import { workspace } from "../state/workspace.svelte";
  let { page }: { page: Page | null } = $props();
  const nodeTypes = { entry: EntryNode };
  let nodes = $state<Node[]>([]);
  let edges = $state<Edge[]>([]);
  let current = $state<string | null>(null);
  $effect(() => {
    if (!page || page.id === current) return;
    current = page.id;
    nodes = page.entries.map((entry) => ({
      id: entry.id,
      type: "entry",
      position: entry.position,
      data: { entry },
    }));
    edges =
      page.pageType === "sequence"
        ? page.connections.map((c) => ({
            id: c.id,
            source: c.source,
            target: c.target,
            type: "smoothstep",
          }))
        : [];
  });
</script>

<section>
  {#if page}<div class="head">
      <div>
        <span>{page.pageType}</span>
        <h1>{page.name}</h1>
      </div>
      <div class="meta">Priority<strong>{page.priority}</strong></div>
    </div>
    <div class="flow">
      <SvelteFlow
        bind:nodes
        bind:edges
        {nodeTypes}
        fitView
        nodesConnectable={page.pageType === "sequence"}
        edgesFocusable={page.pageType === "sequence"}
        onnodeclick={(e) => workspace.selectEntry(page.id, e.detail.node.id)}
        onpaneClick={() => workspace.selectPage(page.id)}
        ><Background
          variant={BackgroundVariant.Dots}
          gap={22}
          size={1}
        /><Controls /><MiniMap /></SvelteFlow
      ><button class="add" onclick={() => workspace.createEntry()}
        >+ Add entry</button
      >
    </div>{:else}<div class="empty">
      <span>Computer</span>
      <h1>No page selected</h1>
    </div>{/if}
</section>

<style>
  section {
    min-width: 0;
    min-height: 0;
    display: flex;
    flex: 1;
    flex-direction: column;
    background: var(--canvas);
  }
  .head {
    height: 76px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 22px;
    border-bottom: 2px solid var(--line);
    background: var(--surface);
  }
  .head span,
  .empty span {
    display: block;
    color: var(--accent);
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 900;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }
  h1 {
    margin: 3px 0 0;
    font-size: 20px;
    font-weight: 900;
    letter-spacing: -0.04em;
  }
  .meta {
    display: grid;
    gap: 3px;
    text-align: right;
    color: var(--text-muted);
    font-family: var(--mono);
    font-size: 10px;
    text-transform: uppercase;
  }
  .meta strong {
    color: var(--text);
    font-size: 14px;
  }
  .flow {
    position: relative;
    flex: 1;
    min-height: 0;
  }
  .add {
    position: absolute;
    z-index: 5;
    right: 18px;
    bottom: 18px;
    height: 40px;
    padding: 0 15px;
    border: 2px solid var(--border);
    background: var(--accent);
    color: #111;
    font-size: 11px;
    font-weight: 900;
    text-transform: uppercase;
    box-shadow: 4px 4px 0 #000;
    cursor: pointer;
  }
  .add:active {
    transform: translate(4px, 4px);
    box-shadow: none;
  }
  .empty {
    display: grid;
    flex: 1;
    place-content: center;
    text-align: center;
  }
  .empty h1 {
    font-size: 32px;
  }
  :global(.svelte-flow) {
    background: var(--canvas);
  }
  :global(.svelte-flow__edge-path) {
    stroke: var(--accent);
    stroke-width: 3;
  }
  :global(.svelte-flow__controls),
  :global(.svelte-flow__minimap) {
    border: 2px solid var(--border) !important;
    border-radius: 0 !important;
    box-shadow: 4px 4px 0 #000;
  }
  :global(.svelte-flow__controls-button) {
    background: var(--surface) !important;
    fill: var(--text) !important;
    border-radius: 0 !important;
  }
</style>
