<script lang="ts">
  import { X } from "lucide-svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { PAGE_TYPES, type PageType, type Value } from "$lib/types/model";
  import Field from "../Field.svelte";

  let { open = $bindable(false) } = $props<{ open?: boolean }>();

  let name = $state<Value>({ text: "Nouvelle page" });
  let pageType = $state<Value>({ enum: PAGE_TYPES[0] });
  let priority = $state<Value>({ integer: 0 });

  function close() {
    open = false;
  }

  function create() {
    if (
      typeof name !== "object" ||
      name === null ||
      !("text" in name) ||
      !name.text.trim()
    )
      return;

    if (
      typeof pageType !== "object" ||
      pageType === null ||
      !("enum" in pageType) ||
      !PAGE_TYPES.includes(pageType.enum as PageType)
    )
      return;

    if (
      typeof priority !== "object" ||
      priority === null ||
      !("integer" in priority)
    )
      return;

    appStore.createPage(
      name.text.trim(),
      pageType.enum as PageType,
      priority.integer,
    );

    close();
  }

  function submit(event: SubmitEvent) {
    event.preventDefault();
    create();
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if (event.key === "Escape") close();
  }}
/>

{#if open}
  <div class="backdrop">
    <button
      type="button"
      class="backdrop-close"
      aria-label="Close"
      tabindex="-1"
      onclick={close}
    ></button>

    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="create-page-title"
      tabindex="-1"
    >
      <div class="header">
        <h2 id="create-page-title">New page</h2>

        <button
          type="button"
          class="close"
          aria-label="Close"
          title="Close"
          onclick={close}
        >
          <X size={16} />
        </button>
      </div>

      <form onsubmit={submit}>
        <Field label="Name" value={name} onchange={(value) => (name = value)} />

        <Field
          label="Type"
          value={pageType}
          options={PAGE_TYPES}
          onchange={(value) => (pageType = value)}
        />

        <Field
          label="Priority"
          value={priority}
          onchange={(value) => (priority = value)}
        />

        <div class="actions">
          <button type="submit" class="btn-brutalist create"> Create </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 1000;
    display: grid;
    place-items: center;
    padding: 24px;
    background: rgba(0, 0, 0, 0.5);
  }

  .backdrop-close {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    padding: 0;
    border: 0;
    background: transparent;
  }

  .dialog {
    position: relative;
    z-index: 1;
    width: min(320px, 100%);
    padding: 20px;
    border: 2px solid var(--accent);
    border-radius: var(--radius);
    background: var(--surface-raised);
    color: var(--text);
    box-shadow: 6px 6px 0 var(--accent-shadow);
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 20px;
  }

  .header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 800;
  }

  .close {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    padding: 0;
    border: 0;
    border-radius: 4px;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
  }

  .close:hover {
    background: rgba(0, 0, 0, 0.05);
    color: var(--text);
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 6px;
  }

  .create {
    min-width: 80px;
    min-height: 32px;
    padding: 0 12px;
    font-size: 13px;
  }
</style>
