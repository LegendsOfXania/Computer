<script lang="ts">
  import { Plus, Route, FileText, Pencil, Trash2 } from "lucide-svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import type { PageInfo } from "$lib/types/model";
  import PageDialog from "./dialogs/Page.svelte";
  import ConfirmDialog from "./dialogs/Confirm.svelte";

  let { hover = $bindable(false) } = $props<{ hover?: boolean }>();

  let dialogOpen = $state(false);
  let editingPage = $state<PageInfo | null>(null);

  let confirmDialogOpen = $state(false);
  let pageToDelete = $state<PageInfo | null>(null);

  let contextMenu = $state<{
    visible: boolean;
    x: number;
    y: number;
    page: PageInfo | null;
  }>({
    visible: false,
    x: 0,
    y: 0,
    page: null,
  });

  const expanded = $derived(hover || contextMenu.visible);

  function openCreate() {
    editingPage = null;
    dialogOpen = true;
  }

  function handleContextMenu(event: MouseEvent, page: PageInfo) {
    event.preventDefault();
    contextMenu = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
      page,
    };
  }

  function closeContextMenu() {
    contextMenu.visible = false;
  }

  function triggerEdit() {
    if (contextMenu.page) {
      editingPage = contextMenu.page;
      dialogOpen = true;
    }
    closeContextMenu();
  }

  function triggerDelete() {
    if (contextMenu.page) {
      pageToDelete = contextMenu.page;
      confirmDialogOpen = true;
    }
    closeContextMenu();
  }

  function handleConfirmDelete() {
    if (pageToDelete) {
      appStore.removePage(pageToDelete.id);
      pageToDelete = null;
    }
  }
</script>

<svelte:window onclick={closeContextMenu} />

<aside
  class="sidebar"
  class:expanded
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
        onclick={openCreate}
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
          oncontextmenu={(event) => handleContextMenu(event, page)}
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

{#if contextMenu.visible}
  <div
    class="context-menu"
    role="menu"
    tabindex="-1"
    style="top: {contextMenu.y}px; left: {contextMenu.x}px;"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => {
      if (e.key === "Escape") closeContextMenu();
    }}
  >
    <button
      type="button"
      class="context-menu-item"
      role="menuitem"
      onclick={triggerEdit}
    >
      <Pencil size={14} />
      <span>Edit</span>
    </button>

    <button
      type="button"
      class="context-menu-item"
      role="menuitem"
      onclick={triggerDelete}
    >
      <Trash2 size={14} />
      <span>Delete</span>
    </button>
  </div>
{/if}

<PageDialog bind:open={dialogOpen} page={editingPage} />

<ConfirmDialog
  bind:open={confirmDialogOpen}
  title="Delete Page"
  message={`Are you sure you want to delete "${pageToDelete?.name ?? ""}"?`}
  confirmLabel="Delete"
  danger={true}
  onconfirm={handleConfirmDelete}
/>

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

  .sidebar.expanded {
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

  .sidebar.expanded .pages-header > span,
  .sidebar.expanded .name {
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

  .context-menu {
    position: fixed;
    z-index: 1000;
    background: var(--surface);
    border: 2px solid var(--accent);
    border-radius: var(--radius);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    padding: 4px;
    min-width: 120px;
  }

  .context-menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 10px;
    border: 0;
    background: transparent;
    color: var(--text);
    font-size: 13px;
    font-weight: 600;
    text-align: left;
    cursor: pointer;
    border-radius: calc(var(--radius) - 2px);
  }

  .context-menu-item:hover {
    background: var(--accent);
    color: var(--on-accent);
  }
</style>
