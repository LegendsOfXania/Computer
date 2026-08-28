<script lang="ts">
  import { Copy, Check, Route, FileText } from "lucide-svelte";
  import { appStore } from "$lib/stores/app.svelte";
  let page = $derived(appStore.selectedPage),
    published = $state(false),
    copied = $state(false);
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
    {#if page}{@const Icon =
        page.page_type === "sequence" ? Route : FileText}<span class="icon"
        ><Icon size={18} /></span
      ><span class="name">{page.name}</span><button class="id" onclick={copy}
        ><span>{page.id}</span>{#if copied}<Check size={12} />{:else}<Copy
            size={12}
          />{/if}</button
      >{:else}<span class="name muted">No page selected</span>{/if}
  </div>
  <div class="actions">
    <div class="connection"><span></span>Online</div>
    <button
      class="btn-brutalist publish"
      class:active={published}
      onclick={() => (published = !published)}
      >{published ? "Published" : "Staging"}</button
    >
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
  .info,
  .actions,
  .connection {
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
  .muted,
  .connection {
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
  .connection {
    gap: 8px;
    font-size: 13px;
    font-weight: 600;
  }
  .connection span {
    width: 10px;
    height: 10px;
    border: 1px solid var(--border);
    background: var(--success);
  }
  .publish {
    min-width: 95px;
    min-height: 34px;
    padding: 0 14px;
    font-size: 12px;
  }
</style>
