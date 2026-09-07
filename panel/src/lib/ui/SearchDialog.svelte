<script lang="ts">
  import { Box, FileText, Plus, Search as SearchIcon, X } from "lucide-svelte";
  import Dialog from "./Dialog.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { defaultSchema, formatEntryTypeName } from "$lib/domain/model";
  import {
    mergeSearchQueries,
    parseSearchQuery,
  } from "$lib/features/search/query";
  import { search } from "$lib/features/search/engine";

  let {
    open = $bindable(false),
    fixedQuery = "",
    onSelectPage,
    onSelectType,
    onSelectEntry,
  }: {
    open?: boolean;
    fixedQuery?: string;
    onSelectPage?: (id: string) => void;
    onSelectType?: (type: string) => void;
    onSelectEntry?: (key: string) => void;
  } = $props();
  let query = $state("");
  let selected = $state(0);
  const parsed = $derived(
    mergeSearchQueries(parseSearchQuery(fixedQuery), parseSearchQuery(query)),
  );
  const results = $derived(
    search(
      parsed,
      appStore.allEntrySummaries,
      appStore.pages,
      appStore.entryDefinitions,
    ),
  );
  const items = $derived([
    ...results.pages.map((page) => ({ kind: "page" as const, id: page.id })),
    ...results.entries.map((entry) => ({
      kind: "entry" as const,
      id: entry.key,
    })),
    ...results.newEntries.map((type) => ({ kind: "new" as const, id: type })),
  ]);

  $effect(() => {
    if (open) {
      query = "";
      selected = 0;
    }
  });
  $effect(() => {
    if (selected >= items.length) selected = Math.max(0, items.length - 1);
  });

  function close() {
    open = false;
  }
  function activate(index = selected) {
    const item = items[index];
    if (!item) return;
    if (item.kind === "page") openPage(item.id);
    else if (item.kind === "entry") openEntry(item.id);
    else createEntry(item.id);
  }
  function keydown(event: KeyboardEvent) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      selected = Math.min(items.length - 1, selected + 1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      selected = Math.max(0, selected - 1);
    } else if (event.key === "Enter") {
      event.preventDefault();
      activate();
    }
  }
  function openPage(id: string) {
    onSelectPage?.(id);
    if (!onSelectPage) appStore.selectPage(id);
    close();
  }
  function openEntry(key: string) {
    onSelectEntry?.(key);
    if (!onSelectEntry) appStore.openReference(key);
    close();
  }
  function createEntry(type: string) {
    if (onSelectType) {
      onSelectType(type);
      close();
      return;
    }
    const definition = appStore.entryDefinitions[type];
    if (!definition || !appStore.selectedPageId) return;
    const fields = Object.fromEntries(
      definition.fields.map((field) => [
        field.name,
        field.name === "name"
          ? { text: formatEntryTypeName(type) }
          : defaultSchema(field.schema),
      ]),
    );
    const key = appStore.createEntry(type, fields);
    if (key) appStore.openReference(key);
    close();
  }
</script>

<Dialog bind:open initialFocus="input" class="search-dialog">
  <div class="search-box">
    <SearchIcon size={16} />
    <input
      bind:value={query}
      onkeydown={keydown}
      type="text"
      placeholder="Search..."
      autocomplete="off"
      spellcheck="false"
    />
    <button type="button" class="close" aria-label="Close" onclick={close}
      ><X size={16} /></button
    >
  </div>
  <div class="results">
    {#each items as item, index (item.kind + item.id)}
      {#if index === 0 || items[index - 1].kind !== item.kind}<div
          class="section-label"
        >
          {item.kind === "page"
            ? "Pages"
            : item.kind === "entry"
              ? "Entries"
              : "Create new"}
        </div>{/if}
      {#if item.kind === "page"}
        {@const page = results.pages.find((value) => value.id === item.id)}
        {#if page}<button
            type="button"
            class="result"
            class:active={selected === index}
            onclick={() => openPage(page.id)}
            ><FileText size={14} />
            <div class="result-info">
              <span class="result-title">{page.name}</span><span
                class="result-type">{page.page_type}</span
              >
            </div></button
          >{/if}
      {:else if item.kind === "entry"}
        {@const entry = results.entries.find((value) => value.key === item.id)}
        {#if entry}<button
            type="button"
            class="result"
            class:active={selected === index}
            onclick={() => openEntry(entry.key)}
            ><Box size={14} />
            <div class="result-info">
              <span class="result-title">{entry.name}</span><span
                class="result-type"
                >{entry.entryType} · {entry.key.slice(
                  0,
                  entry.key.indexOf(":"),
                )}</span
              >
            </div></button
          >{/if}
      {:else}
        <button
          type="button"
          class="result"
          class:active={selected === index}
          onclick={() => createEntry(item.id)}
          ><Plus size={14} />
          <div class="result-info">
            <span class="result-title">{formatEntryTypeName(item.id)}</span
            ><span class="result-type">{item.id}</span>
          </div></button
        >
      {/if}
    {:else}
      <div class="empty-results">No matching result</div>
    {/each}
  </div>
</Dialog>
