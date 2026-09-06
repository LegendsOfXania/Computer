<script lang="ts">
  import {
    Copy,
    Check,
    Trash2,
    CopyPlus,
    FolderInput,
    Replace,
  } from "lucide-svelte";
  import Field from "./Field.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { displayName } from "$lib/types/model";
  import SearchDialog from "./dialogs/Search.svelte";

  let entry = $derived(appStore.selectedEntry),
    copied = $state(false),
    width = $state(320),
    resizing = $state(false),
    showMove = $state(false),
    showReplace = $state(false);

  let page = $derived(appStore.selectedPage);

  const min = 280,
    max = 720;

  async function copy() {
    if (!entry) return;
    try {
      await navigator.clipboard.writeText(entry.id);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch {}
  }

  function resize(e: PointerEvent) {
    e.preventDefault();
    resizing = true;
    const x = e.clientX,
      w = width;
    const move = (ev: PointerEvent) =>
      (width = Math.min(max, Math.max(min, w + x - ev.clientX)));
    const up = () => {
      resizing = false;
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  }

  function remove() {
    if (entry) appStore.deleteEntry(entry.id);
  }

  function duplicate() {
    if (entry) appStore.duplicateEntry(entry.id);
  }

  function move(pageId: string) {
    if (entry) appStore.moveEntry(entry.id, pageId);
  }

  function replace(entryType: string) {
    if (entry && entryType !== entry.entry_type) {
      appStore.replaceEntry(entry.id, entryType);
    }
  }
</script>

<aside
  class="inspector"
  class:open={!!entry}
  class:resizing
  style:width={entry ? `${width}px` : "0px"}
>
  <button
    type="button"
    class="handle"
    aria-label="Redimensionner l'inspecteur"
    onpointerdown={resize}
    ondblclick={() => (width = 320)}
  ></button>

  {#if entry}
    <div class="content" style:width={`${width}px`}>
      <header>
        <h2>{displayName(entry)}</h2>
        <div class="meta">
          <span class="type">{entry.entry_type}</span><button
            class="id"
            onclick={copy}
            ><span>{entry.id}</span>{#if copied}<Check size={12} />{:else}<Copy
                size={12}
              />{/if}</button
          >
        </div>
      </header>

      <div class="fields">
        {#each Object.entries(entry.fields) as [key, value] (key)}<Field
            label={key}
            {value}
            schema={appStore.fieldSchema(entry.entry_type, key)?.schema}
            onchange={(v) => appStore.updateEntryField(entry!.id, key, v)}
          />{/each}
      </div>

      <footer class="toolbar">
        <button type="button" onclick={duplicate}>
          <CopyPlus size={14} />
          <span>Dupliquer</span>
        </button>

        <button type="button" onclick={() => (showMove = true)}>
          <FolderInput size={14} />
          <span>Déplacer</span>
        </button>

        <button type="button" onclick={() => (showReplace = true)}>
          <Replace size={14} />
          <span>Remplacer</span>
        </button>

        <button type="button" onclick={remove} class="danger">
          <Trash2 size={14} />
          <span>Supprimer</span>
        </button>
      </footer>
    </div>
  {/if}
</aside>

<SearchDialog
  bind:open={showMove}
  fixedQuery={page ? `!page !type:${page.page_type}` : "!page"}
  onSelectPage={move}
/>

<SearchDialog
  bind:open={showReplace}
  fixedQuery={page ? `!entry !new !type:${page.page_type}` : "!entry !new"}
  onSelectType={replace}
/>

<style>
  .inspector {
    position: relative;
    display: flex;
    flex-direction: column;
    flex: 0 0 auto;
    height: 100%;
    overflow: hidden;
    background: var(--surface);
    border-left: 0 solid var(--accent);
    opacity: 0;
    transition:
      width var(--sidebar-transition-duration) cubic-bezier(0.4, 0, 0.2, 1),
      border-left-width var(--sidebar-transition-duration),
      opacity 0.2s;
  }
  .inspector.open {
    opacity: 1;
    border-left-width: 2px;
  }
  .inspector.resizing {
    transition: none;
  }
  .handle {
    position: absolute;
    inset: 0 auto 0 -7px;
    width: 14px;
    padding: 0;
    border: 0;
    background: transparent;
    cursor: ew-resize;
    z-index: 2;
    touch-action: none;
  }
  .handle:hover {
    background: var(--accent);
    opacity: 0.4;
  }
  .content {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 100%;
    overflow-y: auto;
    padding: 20px;
    box-sizing: border-box;
  }
  header {
    margin-bottom: 20px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--border-muted);
  }
  h2 {
    margin: 0 0 6px;
    font-size: 20px;
    font-weight: 800;
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
  }
  .type {
    padding: 2px 8px;
    border-radius: var(--radius);
    background: var(--accent);
    color: var(--on-accent);
    font-weight: 800;
    text-transform: uppercase;
  }
  .id {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 2px 6px;
    border: 0;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.05);
    color: var(--text-muted);
    font-family: ui-monospace, monospace;
    font-size: 12px;
    cursor: pointer;
  }
  .id:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text);
  }
  .fields {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .toolbar {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    gap: 6px;
    margin-top: auto;
    padding-top: 24px;
    border-top: 1px solid var(--border-muted);
  }
  .toolbar button {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border: 0;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.03);
    color: var(--text-muted);
    font-size: 13px;
    cursor: pointer;
    white-space: nowrap;
    width: 100%;
  }
  .toolbar button:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text);
  }
  .toolbar button.danger {
    margin-left: 0;
    color: var(--danger, #ff4d4d);
  }
  .toolbar button.danger:hover {
    background: color-mix(in srgb, red 15%, transparent);
    color: red;
  }
</style>
