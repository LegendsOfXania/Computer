<script lang="ts">
  import { Copy, Check, Route, FileText, Search } from "lucide-svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import EntrySearch from "./dialogs/EntrySearch.svelte";

  let page = $derived(appStore.selectedPage),
    published = $state(false),
    copied = $state(false),
    createEntryOpen = $state(false);

  async function copy() {
    if (!page) return;

    try {
      await navigator.clipboard.writeText(page.id);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch {}
  }
</script>

<header class="header">
  <div class="info">
    {#if page}
      {@const Icon = page.page_type === "sequence" ? Route : FileText}
      <span class="icon">
        <Icon size={18} />
      </span>

      <span class="name">{page.name}</span>

      <button class="id" onclick={copy}>
        <span>{page.id}</span>
        {#if copied}
          <Check size={12} />
        {:else}
          <Copy size={12} />
        {/if}
      </button>
    {:else}
      <span class="name muted">No page selected</span>
    {/if}
  </div>

  <div class="actions">
    <button
      class="search"
      type="button"
      disabled={!page}
      onclick={() => (createEntryOpen = true)}
    >
      <Search size={14} />
      <span>Add entry...</span>
    </button>

    <button
      class="btn-brutalist publish"
      class:active={published}
      onclick={() => (published = !published)}
    >
      {published ? "Published" : "Staging"}
    </button>
  </div>
</header>

<EntrySearch bind:open={createEntryOpen} />

<style>
  .header {
    height: var(--header-height);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 15px 0 25px;
    border-bottom: 2px solid var(--accent);
    background: var(--surface);
  }

  .info,
  .actions {
    display: flex;
    align-items: center;
  }

  .info {
    gap: 12px;
  }

  .actions {
    gap: 20px;
  }

  .icon {
    display: flex;
    color: var(--accent);
  }

  .name {
    font-size: 14px;
    font-weight: 700;
  }

  .muted {
    color: var(--text-muted);
  }

  .id {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 2px 6px;
    border: 0;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.08);
    color: var(--text-muted);
    font:
      400 12px ui-monospace,
      monospace;
    cursor: pointer;
  }

  .id:hover {
    color: var(--text);
    background: rgba(0, 0, 0, 0.12);
  }

  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 180px;
    height: 30px;
    padding: 0 10px;

    border: 1px solid var(--border-muted);
    border-radius: 5px;

    background: var(--surface-raised);
    color: var(--text-muted);

    font-size: 12px;
    font-weight: 500;
    text-align: left;

    cursor: pointer;
  }

  .search:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .search:disabled {
    opacity: 0.5;
    cursor: default;
    border-color: var(--border-muted);
    color: var(--text-muted);
  }

  .publish {
    min-width: 95px;
    min-height: 34px;
    padding: 0 14px;
    font-size: 12px;
  }
</style>
