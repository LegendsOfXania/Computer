<script lang="ts">
  import { Plus } from "lucide-svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { PAGE_ICONS } from "$lib/types/pages";

  let { hover = $bindable(false) } = $props<{ hover?: boolean }>();
</script>

<aside
  class="sidebar"
  class:hover
  onmouseenter={() => (hover = true)}
  onmouseleave={() => (hover = false)}
>
  <div class="sidebar-content">
    <div class="pages-header">
      <span class="header-title">PAGES</span>

      <button
        type="button"
        class="btn-brutalist create-page"
        title="Create page"
      >
        <Plus size={16} />
      </button>
    </div>

    <div class="pages">
      {#each appStore.pages as page (page.id)}
        {@const Icon = PAGE_ICONS[page.page_type]}

        <button
          type="button"
          class="page"
          class:selected={page.id === appStore.selectedPageId}
          onclick={() => appStore.selectPage(page.id)}
          title={page.name}
        >
          <div class="icon-wrapper">
            <Icon size={16} />
          </div>

          <span class="page-name">
            {page.name}
          </span>
        </button>
      {/each}
    </div>
  </div>
</aside>

<style>
  .sidebar {
    position: fixed;
    left: 0;
    top: 0;
    bottom: 0;

    width: var(--sidebar-width-collapsed);

    background: var(--surface);
    border-right: 2px solid var(--accent);

    cursor: pointer;

    transition: width var(--sidebar-transition-duration)
      cubic-bezier(0.4, 0, 0.2, 1);
    overflow: hidden;

    z-index: 100;
  }

  .sidebar:hover {
    width: var(--sidebar-width-expanded);
  }

  .sidebar-content {
    box-sizing: border-box;
    width: var(--sidebar-width-expanded);
    padding: 32px 10px;
  }

  .pages-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 32px;

    margin-bottom: 12px;
    padding-bottom: 8px;

    color: var(--text-muted);

    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.12em;
  }

  .header-title {
    opacity: 0;
    white-space: nowrap;
    transition: opacity 0.2s ease;
  }

  .sidebar:hover .header-title {
    opacity: 1;
  }

  .create-page {
    width: 28px;
    height: 28px;
    padding: 0;
    flex-shrink: 0;
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

    border: none;
    border-radius: var(--radius);

    background: transparent;
    color: var(--text-muted);

    text-align: left;

    cursor: pointer;

    transition:
      background-color 0.15s ease,
      color 0.15s ease;
  }

  .icon-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    flex-shrink: 0;
  }

  .page-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;

    font-size: 13px;
    font-weight: 700;

    opacity: 0;
    transition: opacity 0.2s ease;
  }

  .sidebar:hover .page-name {
    opacity: 1;
  }

  .page:hover {
    background: rgba(0, 0, 0, 0.05);
    color: var(--text);
  }

  .page.selected {
    background: var(--accent);
    color: var(--on-accent);
  }
</style>
