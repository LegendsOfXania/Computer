<script lang="ts">
  import { Search, X, ChevronRight } from "lucide-svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { defaultSchema, formatEntryTypeName } from "$lib/types/model";

  let {
    open = $bindable(false),
    requiredTags,
  }: {
    open?: boolean;
    requiredTags?: string[];
  } = $props();

  let query = $state("");
  let input: HTMLInputElement | undefined = $state();

  const effectiveTags = $derived(
    requiredTags ??
      (appStore.selectedPage ? [appStore.selectedPage.page_type] : []),
  );

  const matchingTypes = $derived(
    Object.entries(appStore.entryDefinitions)
      .filter(
        ([, definition]) =>
          effectiveTags.length === 0 ||
          effectiveTags.some((tag) => definition.tags.includes(tag)),
      )
      .map(([type]) => type)
      .filter((type) =>
        type.toLowerCase().includes(query.trim().toLowerCase()),
      ),
  );

  $effect(() => {
    if (open) {
      query = "";
      queueMicrotask(() => input?.focus());
    }
  });

  function close() {
    open = false;
  }

  function create(entryType: string) {
    const definition = appStore.entryDefinitions[entryType];
    if (!definition) return;

    const fields = Object.fromEntries(
      definition.fields.map((field) => [
        field.name,
        field.name === "name"
          ? { text: formatEntryTypeName(entryType) }
          : defaultSchema(field.schema),
      ]),
    );

    const key = appStore.createEntry(entryType, fields);
    if (key) appStore.openReference(key);

    close();
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if (event.key === "Escape") close();
  }}
/>

{#if open}
  <div class="backdrop">
    <button
      type="button"
      class="backdrop-close"
      aria-label="Close"
      tabindex="-1"
      onclick={close}
    ></button>

    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="entry-search-title"
      tabindex="-1"
    >
      <div class="search-box">
        <Search size={16} />
        <input
          bind:this={input}
          type="text"
          placeholder="Search an entry type to create..."
          bind:value={query}
        />
        <button type="button" class="close" aria-label="Close" onclick={close}>
          <X size={16} />
        </button>
      </div>

      <div class="results">
        {#each matchingTypes as type (type)}
          <button type="button" class="result" onclick={() => create(type)}>
            <div class="result-info">
              <span class="result-title">{formatEntryTypeName(type)}</span>
              <span class="result-type">{type}</span>
            </div>
            <ChevronRight size={14} class="result-icon" />
          </button>
        {:else}
          <div class="empty-results">No matching entry type</div>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 1000;
    display: grid;
    place-items: start center;
    padding: 24px;
    padding-top: 15vh;
    background: rgba(0, 0, 0, 0.5);
  }

  .backdrop-close {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    padding: 0;
    border: 0;
    background: transparent;
  }

  .dialog {
    position: relative;
    z-index: 1;
    width: min(420px, 100%);
    max-height: min(400px, 70vh);
    display: flex;
    flex-direction: column;
    border: 2px solid var(--accent);
    border-radius: var(--radius);
    background: var(--surface-raised);
    color: var(--text);
    box-shadow: 6px 6px 0 var(--accent-shadow);
    overflow: hidden;
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border-muted);
    color: var(--text-muted);
  }

  .search-box input {
    flex: 1;
    border: 0;
    background: transparent;
    color: var(--text);
    font-size: 14px;
    outline: none;
  }

  .close {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border: 0;
    border-radius: 4px;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
  }

  .close:hover {
    background: rgba(0, 0, 0, 0.05);
    color: var(--text);
  }

  .results {
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .result {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 8px 12px;
    border: 1px solid transparent;
    border-radius: calc(var(--radius) - 2px);
    background: transparent;
    color: var(--text);
    text-align: left;
    cursor: pointer;
    transition:
      background 0.15s ease,
      border-color 0.15s ease,
      transform 0.1s ease;
  }

  .result-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .result-title {
    font-size: 13px;
    font-weight: 600;
    line-height: 1.2;
  }

  .result-type {
    font-size: 11px;
    font-family: monospace;
    color: var(--text-muted);
    opacity: 0.8;
  }

  :global(.result-icon) {
    color: var(--text-muted);
    opacity: 0;
    transform: translateX(-4px);
    transition:
      opacity 0.15s ease,
      transform 0.15s ease;
  }

  .result:hover {
    background: var(--surface);
    border-color: var(--border-muted);
  }

  .result:hover :global(.result-icon) {
    opacity: 1;
    transform: translateX(0);
  }

  .result:active {
    background: var(--accent);
    color: var(--surface-raised);
    border-color: var(--accent);
  }

  .result:active .result-type,
  .result:active :global(.result-icon) {
    color: var(--surface-raised);
    opacity: 0.9;
  }

  .empty-results {
    padding: 16px;
    text-align: center;
    color: var(--text-muted);
    font-size: 13px;
  }
</style>
