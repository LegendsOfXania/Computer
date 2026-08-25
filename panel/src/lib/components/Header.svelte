<script lang="ts">
  import { Copy } from "lucide-svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { PAGE_ICONS } from "$lib/types/pages";

  let page = $derived(appStore.selectedPage);
  let Icon = $derived(page ? PAGE_ICONS[page.page_type] : null);

  // Placeholder local state until the panel actually talks to a server.
  let isPublished = $state(false);

  async function copyPageId() {
    if (page === null) return;
    await navigator.clipboard.writeText(page.id);
  }

  function togglePublish() {
    isPublished = !isPublished;
  }
</script>

<header class="header">
  <div class="informations">
    {#if page !== null}
      <span class="page-icon">
        {#if Icon}
          <Icon size={18} />
        {/if}
      </span>
      <span class="page-name">{page.name}</span>
      <button type="button" class="page-id" onclick={copyPageId}>
        <span>{page.id}</span>
        <Copy size={12} />
      </button>
    {:else}
      <span class="page-name muted">No page selected</span>
    {/if}
  </div>

  <div class="actions">
    <div class="connection">
      <span class="status connected"></span>
      <span>Online</span>
    </div>

    <button
      class="btn-brutalist publish"
      class:active={isPublished}
      onclick={togglePublish}
    >
      <span>{isPublished ? "Published" : "Staging"}</span>
    </button>
  </div>
</header>

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

  .informations {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .page-icon {
    display: flex;
    align-items: center;
    color: var(--accent);
  }

  .page-name {
    font-weight: 700;
    font-size: 14px;
    color: var(--text);
  }

  .page-name.muted {
    color: var(--text-muted);
    font-weight: 600;
  }

  .page-id {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 2px 6px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.08);
    color: var(--text-muted);
    font-size: 12px;
    font-family: monospace;
    font-weight: 400;
    cursor: pointer;
    transition: background 0.15s ease;
    border: none;
  }

  .page-id:hover {
    background: rgba(0, 0, 0, 0.12);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 20px;
  }

  .connection {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-muted);
    font-size: 13px;
    font-weight: 600;
  }

  .status {
    width: 10px;
    height: 10px;
    border: 1px solid var(--border);
    background: var(--danger);
  }

  .status.connected {
    background: var(--success);
  }

  .publish {
    min-height: 34px;
    min-width: 95px;
    padding: 0 14px;
    font-size: 12px;
  }
</style>
