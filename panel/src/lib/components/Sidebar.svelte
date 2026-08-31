<script lang="ts">
  import { Plus, Route, FileText } from "lucide-svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import CreatePage from "./dialogs/CreatePage.svelte";

  let { hover = $bindable(false) } = $props<{ hover?: boolean }>();

  let createOpen = $state(false);
</script>

<aside
  class="sidebar"
  onmouseenter={() => (hover = true)}
  onmouseleave={() => (hover = false)}
>
  <div class="content">
    <div class="pages-header">
      <span>PAGES</span>

      <button
        type="button"
        class="btn-brutalist create"
        title="Create page"
        aria-label="Create page"
        onclick={() => (createOpen = true)}
      >
        <Plus size={16} />
      </button>
    </div>

    <div class="pages">
      {#each appStore.pages as page (page.id)}
        {@const Icon = page.page_type === "sequence" ? Route : FileText}

        <button
          type="button"
          class:selected={page.id === appStore.selectedPageId}
          class="page"
          onclick={() => appStore.selectPage(page.id)}
          title={page.name}
        >
          <span class="icon">
            <Icon size={16} />
          </span>

          <span class="name">{page.name}</span>
        </button>
      {/each}
    </div>
  </div>
</aside>

<CreatePage bind:open={createOpen} />

<style>
  .sidebar {
    position: fixed;
    inset: 0 auto 0 0;
    width: var(--sidebar-width-collapsed);
    overflow: hidden;
    background: var(--surface);
    border-right: 2px solid var(--accent);
    transition: width var(--sidebar-transition-duration)
      cubic-bezier(0.4, 0, 0.2, 1);
    z-index: 100;
  }

  .sidebar:hover {
    width: var(--sidebar-width-expanded);
  }

  .content {
    width: var(--sidebar-width-expanded);
    padding: 32px 10px;
  }

  .pages-header {
    position: relative;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
    padding-bottom: 8px;
    color: var(--text-muted);
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.12em;
  }

  .pages-header > span,
  .name {
    opacity: 0;
    white-space: nowrap;
    transition: opacity 0.2s;
  }

  .sidebar:hover .pages-header > span,
  .sidebar:hover .name {
    opacity: 1;
  }

  .create {
    width: 28px;
    height: 28px;
    padding: 0;
  }

  .pages {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .page {
    display: flex;
    align-items: center;
    width: 100%;
    height: 36px;
    padding: 0;
    border: 0;
    border-radius: var(--radius);
    background: transparent;
    color: var(--text-muted);
    text-align: left;
    cursor: pointer;
  }

  .page:hover {
    background: rgba(0, 0, 0, 0.05);
    color: var(--text);
  }

  .page.selected {
    background: var(--accent);
    color: var(--on-accent);
  }

  .icon {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    flex: 0 0 auto;
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 13px;
    font-weight: 700;
  }
</style>
