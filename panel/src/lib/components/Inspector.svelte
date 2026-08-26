<script lang="ts">
  import { appStore } from "$lib/stores/app.svelte";
  import { displayName } from "$lib/types/model";
  import Field from "./Field.svelte";

  let entry = $derived(appStore.selectedEntry);
</script>

{#if entry !== null}
  <aside class="inspector">
    <header>
      <h2>{displayName(entry)}</h2>
      <div class="meta">
        <span class="entry-type">{entry.entry_type}</span>
        <span class="entry-id">{entry.id}</span>
      </div>
    </header>

    <div class="fields">
      {#each Object.entries(entry.fields) as [key, value] (key)}
        <Field label={key} {value} />
      {/each}
    </div>
  </aside>
{/if}

<style>
  .inspector {
    width: 320px;
    flex-shrink: 0;

    height: 100%;
    overflow-y: auto;

    padding: 20px;

    background: var(--surface);
    border-left: 2px solid var(--accent);
  }

  header {
    margin-bottom: 20px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--border-muted);
  }

  h2 {
    margin: 0 0 6px;
    font-size: 20px;
    font-weight: 800;
    color: var(--text);
  }

  .meta {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
  }

  .entry-type {
    padding: 2px 8px;
    border-radius: var(--radius);
    background: var(--accent);
    color: var(--on-accent);
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .entry-id {
    color: var(--text-muted);
    font-family: monospace;
  }

  .fields {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
</style>
