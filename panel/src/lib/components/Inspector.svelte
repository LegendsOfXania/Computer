<script lang="ts">
  import { appStore } from "$lib/stores/app.svelte";
  import { displayName } from "$lib/types/model";
  import { Copy, Check } from "lucide-svelte";
  import Field from "./Field.svelte";

  let entry = $derived(appStore.selectedEntry);
  let copied = $state(false);

  const MIN_WIDTH = 280;
  const MAX_WIDTH = 720;
  const DEFAULT_WIDTH = 320;

  let width = $state(DEFAULT_WIDTH);
  let resizing = $state(false);

  async function copyEntryId() {
    if (entry === null) return;
    await navigator.clipboard.writeText(entry.id);
    copied = true;
    setTimeout(() => {
      copied = false;
    }, 1500);
  }

  function startResize(e: PointerEvent) {
    e.preventDefault();
    resizing = true;
    const startX = e.clientX;
    const startWidth = width;

    function onPointerMove(ev: PointerEvent) {
      const delta = startX - ev.clientX;
      width = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, startWidth + delta));
    }

    function onPointerUp() {
      resizing = false;
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerup", onPointerUp);
    }

    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
  }

  function resetWidth() {
    width = DEFAULT_WIDTH;
  }
</script>

<aside
  class="inspector"
  class:open={entry !== null}
  class:resizing
  style:width={entry !== null ? `${width}px` : "0px"}
>
  <button
    type="button"
    class="resize-handle"
    aria-label="Redimensionner l'inspecteur"
    onpointerdown={startResize}
    ondblclick={resetWidth}
  ></button>

  <div class="inspector-content" style:width={`${width}px`}>
    {#if entry !== null}
      <header>
        <h2>{displayName(entry)}</h2>
        <div class="meta">
          <span class="entry-type">{entry.entry_type}</span>
          <button
            type="button"
            class="entry-id"
            onclick={copyEntryId}
            title="Copy ID"
          >
            <span>{entry.id}</span>
            {#if copied}
              <Check size={12} />
            {:else}
              <Copy size={12} />
            {/if}
          </button>
        </div>
      </header>

      <div class="fields">
        {#each Object.entries(entry.fields) as [key, value] (key)}
          <Field
            label={key}
            {value}
            onchange={(newValue) =>
              appStore.updateEntryField(entry.id, key, newValue)}
          />
        {/each}
      </div>
    {/if}
  </div>
</aside>

<style>
  .inspector {
    position: relative;
    flex-shrink: 0;

    height: 100%;
    overflow: hidden;

    background: var(--surface);
    border-left: 0px solid var(--accent);

    transition:
      width var(--sidebar-transition-duration, 0.3s)
        cubic-bezier(0.4, 0, 0.2, 1),
      border-left-width var(--sidebar-transition-duration, 0.3s) ease,
      opacity 0.2s ease;
    opacity: 0;
  }

  .inspector.open {
    opacity: 1;
    border-left-width: 2px;
  }

  .inspector.resizing {
    transition: none;
  }

  .resize-handle {
    position: absolute;
    top: 0;
    left: -7px;
    width: 14px;
    height: 100%;
    padding: 0;
    margin: 0;
    border: none;
    background: transparent;
    cursor: ew-resize;
    z-index: 2;
    touch-action: none;
  }

  .resize-handle::after {
    content: "";
    position: absolute;
    top: 0;
    left: 50%;
    transform: translateX(-50%);
    width: 2px;
    height: 100%;
    background: var(--accent);
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .resize-handle:hover::after,
  .inspector.resizing .resize-handle::after {
    opacity: 0.6;
  }

  .resize-handle:hover,
  .inspector.resizing .resize-handle {
    background: var(--accent);
    opacity: 0.4;
  }

  .inspector-content {
    box-sizing: border-box;
    height: 100%;
    overflow-y: auto;
    padding: 20px;
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
    color: var(--text);
  }

  .meta {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
  }

  .entry-type {
    padding: 2px 8px;
    border-radius: var(--radius);
    background: var(--accent);
    color: var(--on-accent);
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .entry-id {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 2px 6px;
    border: none;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.05);
    color: var(--text-muted);
    font-size: 12px;
    font-family: ui-monospace, "SF Mono", "Cascadia Code", Menlo, Consolas,
      monospace;
    font-weight: 400;
    cursor: pointer;
    transition:
      background 0.15s ease,
      color 0.15s ease;
  }

  .entry-id:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text);
  }

  .fields {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
</style>
