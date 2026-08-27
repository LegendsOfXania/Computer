<script lang="ts">
  import { appStore } from "$lib/stores/app.svelte";
  import { displayName } from "$lib/types/model";
  import { Copy, Check } from "lucide-svelte";
  import Field from "./Field.svelte";

  let entry = $derived(appStore.selectedEntry);
  let copied = $state(false);

  async function copyEntryId() {
    if (entry === null) return;
    await navigator.clipboard.writeText(entry.id);
    copied = true;
    setTimeout(() => {
      copied = false;
    }, 1500);
  }
</script>

<aside class="inspector" class:open={entry !== null}>
  <div class="inspector-content">
    {#if entry !== null}
      <header>
        <h2>{displayName(entry)}</h2>
        <div class="meta">
          <span class="entry-type">{entry.entry_type}</span>
          <button
            type="button"
            class="entry-id"
            onclick={copyEntryId}
            title="Copy ID"
          >
            <span>{entry.id}</span>
            {#if copied}
              <Check size={12} />
            {:else}
              <Copy size={12} />
            {/if}
          </button>
        </div>
      </header>

      <div class="fields">
        {#each Object.entries(entry.fields) as [key, value] (key)}
          <Field
            label={key}
            {value}
            onchange={(newValue) =>
              appStore.updateEntryField(entry.id, key, newValue)}
          />
        {/each}
      </div>
    {/if}
  </div>
</aside>

<style>
  .inspector {
    width: 0;
    opacity: 0;
    flex-shrink: 0;

    height: 100%;
    overflow: hidden;

    background: var(--surface);
    border-left: 0px solid var(--accent);

    transition:
      width var(--sidebar-transition-duration, 0.3s)
        cubic-bezier(0.4, 0, 0.2, 1),
      border-left-width var(--sidebar-transition-duration, 0.3s) ease,
      opacity 0.2s ease;
  }

  .inspector.open {
    width: 320px;
    opacity: 1;
    border-left-width: 2px;
  }

  .inspector-content {
    box-sizing: border-box;
    width: 320px;
    height: 100%;
    overflow-y: auto;
    padding: 20px;
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
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 2px 6px;
    border: none;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.08);
    color: var(--text-muted);
    font-size: 12px;
    font-family: monospace;
    font-weight: 400;
    cursor: pointer;
    transition:
      background 0.15s ease,
      color 0.15s ease;
  }

  .entry-id:hover {
    background: rgba(0, 0, 0, 0.12);
    color: var(--text);
  }

  .fields {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
</style>
