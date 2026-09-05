<script lang="ts">
  import { Box, FileText, Plus, Search, X } from "lucide-svelte";

  import { appStore } from "$lib/stores/app.svelte";
  import { defaultSchema, formatEntryTypeName } from "$lib/types/model";
  import { mergeSearchQueries, parseSearchQuery } from "../search/query";
  import { search } from "../search/search";

  let {
    open = $bindable(false),
    fixedQuery = "",
  }: {
    open?: boolean;
    fixedQuery?: string;
  } = $props();

  let query = $state("");
  let dialog: HTMLDialogElement;
  let input: HTMLInputElement;

  const parsedQuery = $derived(
    mergeSearchQueries(parseSearchQuery(fixedQuery), parseSearchQuery(query)),
  );

  const results = $derived(search(parsedQuery));

  const hasResults = $derived(
    results.pages.length > 0 ||
      results.entries.length > 0 ||
      results.newEntries.length > 0,
  );

  $effect(() => {
    if (open) {
      query = "";

      if (!dialog.open) {
        dialog.showModal();
      }

      queueMicrotask(() => input?.focus());
    } else if (dialog.open) {
      dialog.close();
    }
  });

  function close() {
    open = false;
  }

  function openPage(id: string) {
    appStore.selectPage(id);
    close();
  }

  function openEntry(key: string) {
    appStore.openReference(key);
    close();
  }

  function createEntry(type: string) {
    const definition = appStore.entryDefinitions[type];

    if (!definition) return;

    const fields = Object.fromEntries(
      definition.fields.map((field) => [
        field.name,
        field.name === "name"
          ? {
              text: formatEntryTypeName(type),
            }
          : defaultSchema(field.schema),
      ]),
    );

    const key = appStore.createEntry(type, fields);

    if (key) {
      appStore.openReference(key);
    }

    close();
  }

  function entryName(entry: { id: string; fields: Record<string, unknown> }) {
    const name = entry.fields.name;

    return name &&
      typeof name === "object" &&
      name !== null &&
      "text" in name &&
      typeof name.text === "string"
      ? name.text
      : entry.id;
  }

  function pageName(pageId: string) {
    return appStore.pages.find((page) => page.id === pageId)?.name ?? pageId;
  }
</script>

<dialog
  bind:this={dialog}
  class="dialog"
  onclose={() => (open = false)}
  onclick={(event) => {
    if (event.target === dialog) {
      close();
    }
  }}
>
  <div class="search-box">
    <Search size={16} />

    <input
      bind:this={input}
      bind:value={query}
      type="text"
      placeholder="Search..."
      autocomplete="off"
      spellcheck="false"
    />

    <button type="button" class="close" aria-label="Close" onclick={close}>
      <X size={16} />
    </button>
  </div>

  <div class="results">
    {#if results.pages.length}
      <div class="section-label">Pages</div>

      {#each results.pages as page (page.id)}
        <button type="button" class="result" onclick={() => openPage(page.id)}>
          <FileText size={14} />

          <div class="result-info">
            <span class="result-title">
              {page.name}
            </span>

            <span class="result-type">
              {page.page_type}
            </span>
          </div>
        </button>
      {/each}
    {/if}

    {#if results.entries.length}
      <div class="section-label">Entries</div>

      {#each results.entries as result (result.key)}
        <button
          type="button"
          class="result"
          onclick={() => openEntry(result.key)}
        >
          <Box size={14} />

          <div class="result-info">
            <span class="result-title">
              {entryName(result.entry)}
            </span>

            <span class="result-type">
              {result.entry.entry_type}
              ·
              {pageName(result.pageId)}
            </span>
          </div>
        </button>
      {/each}
    {/if}

    {#if results.newEntries.length}
      <div class="section-label">Create new</div>

      {#each results.newEntries as type (type)}
        <button type="button" class="result" onclick={() => createEntry(type)}>
          <Plus size={14} />

          <div class="result-info">
            <span class="result-title">
              {formatEntryTypeName(type)}
            </span>

            <span class="result-type">
              {type}
            </span>
          </div>
        </button>
      {/each}
    {/if}

    {#if !hasResults}
      <div class="empty-results">No matching result</div>
    {/if}
  </div>
</dialog>

<style>
  .dialog {
    width: min(460px, 100%);
    max-height: min(440px, 70vh);
    padding: 0;
    border: 2px solid var(--accent);
    border-radius: var(--radius);
    background: var(--surface-raised);
    color: var(--text);
    box-shadow: 6px 6px 0 var(--accent-shadow);
    overflow: hidden;
  }

  .dialog[open] {
    display: flex;
    flex-direction: column;
  }

  .dialog::backdrop {
    background: rgba(0, 0, 0, 0.5);
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
    min-width: 0;
    border: 0;
    outline: none;
    background: transparent;
    color: var(--text);
    font-size: 14px;
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
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px;
    overflow-y: auto;
  }

  .section-label {
    padding: 8px 8px 4px;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
  }

  .result {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 12px;
    border: 1px solid transparent;
    border-radius: calc(var(--radius) - 2px);
    background: transparent;
    color: var(--text);
    text-align: left;
    cursor: pointer;
  }

  .result:hover {
    background: var(--surface);
    border-color: var(--border-muted);
  }

  .result:active {
    background: var(--accent);
    color: var(--surface-raised);
    border-color: var(--accent);
  }

  .result:active .result-type {
    color: var(--surface-raised);
    opacity: 0.9;
  }

  .result-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .result-title {
    font-size: 13px;
    font-weight: 600;
    line-height: 1.2;
  }

  .result-type {
    overflow: hidden;
    font-family: monospace;
    font-size: 11px;
    color: var(--text-muted);
    opacity: 0.8;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .empty-results {
    padding: 16px;
    text-align: center;
    color: var(--text-muted);
    font-size: 13px;
  }
</style>
