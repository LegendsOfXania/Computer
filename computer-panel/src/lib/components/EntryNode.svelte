<script lang="ts">
  import {
    Handle,
    Position,
    type Node,
    type NodeProps,
  } from '@xyflow/svelte';

  import { MoreHorizontal } from 'lucide-svelte';

  import type { Entry } from '../types';

  type EntryNode = Node<
    {
      entry: Entry;
    },
    'entry'
  >;

  let {
    data,
    selected = false,
  }: NodeProps<EntryNode> = $props();

  const entry = $derived(data.entry);
</script>

<Handle
  type="target"
  position={Position.Left}
  class="handle input"
/>

<div class:selected class="node">
  <div class="nh">
    <span>{entry.data.entryType}</span>

    <MoreHorizontal size={16} />
  </div>

  <div class="body">
    {#each Object.entries(entry.data.fields).slice(0, 3) as [key, value]}
      <div>
        <span>{key}</span>
        <strong>{String(value)}</strong>
      </div>
    {/each}
  </div>
</div>

<Handle
  type="source"
  position={Position.Right}
  class="handle output"
/>

<style>
  .node {
    width: 230px;
    border: 2px solid var(--border);
    background: var(--surface);
    box-shadow: 5px 5px 0 #000;
  }

  .node.selected {
    border-color: var(--accent);
    box-shadow: 5px 5px 0 var(--accent);
  }

  .nh {
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 10px;
    border-bottom: 2px solid var(--border);
    background: var(--surface-raised);
    font-family: var(--mono);
    font-size: 12px;
    font-weight: 800;
  }

  .selected .nh {
    background: var(--accent);
    color: #111;
  }

  .body {
    padding: 10px;
  }

  .body div {
    display: grid;
    grid-template-columns: 72px 1fr;
    gap: 10px;
    padding: 5px 0;
    color: var(--text-muted);
    font-family: var(--mono);
    font-size: 11px;
  }

  .body strong {
    overflow: hidden;
    color: var(--text);
    font-weight: 500;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  :global(.handle) {
    width: 12px;
    height: 12px;
    border: 2px solid var(--border);
    border-radius: 0;
    background: var(--control);
  }

  :global(.output) {
    background: var(--accent);
  }
</style>