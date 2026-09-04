<script lang="ts">
  import { untrack } from "svelte";
  import { Background, SvelteFlow, type Edge, type Node } from "@xyflow/svelte";
  import "@xyflow/svelte/dist/base.css";
  import {
    layoutSequenceEntries,
    layoutStaticEntries,
  } from "$lib/editor/layout";
  import { appStore } from "$lib/stores/app.svelte";
  import { displayName } from "$lib/types/model";
  import EntrySearch from "./dialogs/EntrySearch.svelte";

  let nodes = $state.raw<Node[]>([]);
  let edges = $state.raw<Edge[]>([]);
  let createEntryOpen = $state(false);

  function externalLabel(key: string): string {
    appStore.requestEntry(key);

    const entry = appStore.getEntryData(key);
    const pageId = key.slice(0, key.indexOf(":"));
    const pageName =
      appStore.pages.find((p) => p.id === pageId)?.name ?? pageId;

    return entry ? `${pageName} › ${displayName(entry)}` : `${pageName} › ...`;
  }

  $effect(() => {
    const page = appStore.selectedPage;

    if (!page) {
      nodes = [];
      edges = [];
      return;
    }

    const layout =
      page.page_type === "sequence"
        ? layoutSequenceEntries(page.id, appStore.entries, externalLabel)
        : layoutStaticEntries(appStore.entries);

    const selected = appStore.selectedEntryId;

    nodes = layout.nodes.map((node) => ({
      ...node,
      selected: node.id === selected,
    }));
    edges = layout.edges;
  });

  $effect(() => {
    const selected = appStore.selectedEntryId;

    untrack(() => {
      nodes = nodes.map((node) =>
        node.selected === (node.id === selected)
          ? node
          : { ...node, selected: node.id === selected },
      );
    });
  });

  function click({ node }: { node: Node }) {
    if (node.id.includes(":")) {
      appStore.openReference(node.id);
      return;
    }

    appStore.selectEntry(node.id);
  }
</script>

<div class="editor">
  <SvelteFlow
    bind:nodes
    bind:edges
    fitView
    edgesFocusable={false}
    onnodeclick={click}
    onpaneclick={() => appStore.clearEntrySelection()}
  >
    <Background gap={20} />
  </SvelteFlow>

  {#if nodes.length === 0 && appStore.selectedPage}
    <div class="empty">
      <span>There's not much here...</span>
      <button type="button" onclick={() => (createEntryOpen = true)}>
        Add entry
      </button>
    </div>
  {/if}
</div>

<EntrySearch bind:open={createEntryOpen} />

<style>
  .editor {
    position: relative;
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

  :global(.editor .svelte-flow__node.external) {
    background: var(--surface-raised) !important;
    color: var(--text) !important;
    border: 1px dashed var(--border-muted) !important;
    box-shadow: none !important;
    font-weight: 600;
  }

  :global(.editor .svelte-flow__node.external:hover) {
    border-color: var(--accent) !important;
  }

  :global(.editor .svelte-flow__handle:not(.connectable)) {
    display: none;
  }

  :global(.editor .svelte-flow__edge) {
    pointer-events: none;
  }

  .empty {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    pointer-events: none;
  }

  .empty span {
    color: var(--text-muted);
    font-size: 13px;
  }

  .empty button {
    padding: 7px 14px;
    border: 1px solid var(--border-muted);
    border-radius: var(--radius);
    color: var(--text-muted);
    background: var(--surface-raised);
    font: inherit;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    pointer-events: auto;
  }

  .empty button:hover {
    border-color: var(--accent);
    color: var(--accent);
  }
</style>
