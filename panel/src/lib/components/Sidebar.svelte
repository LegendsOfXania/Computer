<script lang="ts">
  import { Plus } from "lucide-svelte";
  import { mockPages } from "$lib/mocks/pages";
  import { PAGE_ICONS, type Page } from "$lib/types/pages";

  let {
    hover = $bindable(false),
    selectedPageId = $bindable<string | null>(mockPages[0]?.id ?? null),
  } = $props<{ hover?: boolean; selectedPageId?: string | null }>();

  function selectPage(pageId: string) {
    selectedPageId = pageId;
  }
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

      <button type="button" class="create-page" title="Create page">
        <Plus size={16} />
      </button>
    </div>

    <div class="pages">
      {#each mockPages as page}
        {@const Icon = PAGE_ICONS[page.page_type]}

        <button
          type="button"
          class="page"
          class:selected={page.id === selectedPageId}
          onclick={() => selectPage(page.id)}
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

    width: 50px;

    background: var(--surface);
    border-right: 2px solid #f97316;

    cursor: pointer;

    transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    overflow: hidden;

    z-index: 100;
  }

  .sidebar:hover {
    width: 250px;
  }

  .sidebar-content {
    box-sizing: border-box;
    width: 250px;
    padding: 16px 10px;
  }

  .pages-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 32px;

    margin-bottom: 12px;
    padding-bottom: 8px;

    border-bottom: 2px solid #f97316;

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
    display: flex;
    align-items: center;
    justify-content: center;

    width: 28px;
    height: 28px;
    padding: 0;
    flex-shrink: 0;

    border: none;
    border-radius: var(--radius);

    background: #f97316;
    color: #1c1917;

    cursor: pointer;

    transform: translate(-1px, -1px);
    box-shadow: 2px 2px 0px #c2410c;

    transition:
      transform 0.15s ease,
      box-shadow 0.15s ease,
      background-color 0.15s ease;
  }

  .create-page:hover {
    background: #fb923c;
    transform: translate(-2px, -2px);
    box-shadow: 3px 3px 0px #c2410c;
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
    background: #f97316;
    color: #1c1917;
  }
</style>
