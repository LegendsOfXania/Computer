<script lang="ts">
  import { X } from "lucide-svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import {
    PAGE_TYPES,
    type PageInfo,
    type PageType,
    type Schema,
    type Value,
  } from "$lib/types/model";
  import Field from "../Field.svelte";
  import { tick } from "svelte";

  let {
    open = $bindable(false),
    page = null,
  }: {
    open?: boolean;
    page?: PageInfo | null;
  } = $props();

  const pageTypeSchema: Schema = { enumeration: PAGE_TYPES };

  let dialog: HTMLDialogElement;

  let name = $state("");
  let pageType = $state<PageType>(PAGE_TYPES[0]);
  let priority = $state(0);

  function resetForm() {
    name = page?.name ?? "New page";
    pageType = page?.page_type ?? PAGE_TYPES[0];
    priority = page?.priority ?? 0;
  }

  $effect(() => {
    if (open) {
      resetForm();
      dialog.showModal();
      tick().then(() => {
        dialog.querySelector<HTMLInputElement>("input")?.select();
      });
    } else {
      dialog.close();
    }
  });

  function submit(event: SubmitEvent) {
    event.preventDefault();
    const trimmed = name.trim();
    if (!trimmed) return;

    if (page) {
      appStore.editPage(page.id, trimmed, priority);
    } else {
      appStore.createPage(trimmed, pageType, priority);
    }
    open = false;
  }
</script>

<dialog
  bind:this={dialog}
  class="dialog"
  onclose={() => (open = false)}
  onclick={(e) => e.target === dialog && (open = false)}
>
  <div class="header">
    <h2>{page ? "Edit page" : "New page"}</h2>
    <button
      type="button"
      class="close"
      aria-label="Close"
      onclick={() => (open = false)}
    >
      <X size={16} />
    </button>
  </div>

  <form onsubmit={submit}>
    <Field
      label="Name"
      value={{ text: name }}
      onchange={(v: Value) => (name = "text" in v ? v.text : name)}
    />

    {#if !page}
      <Field
        label="Type"
        value={{ enum: pageType }}
        schema={pageTypeSchema}
        onchange={(v: Value) =>
          (pageType = "enum" in v ? (v.enum as PageType) : pageType)}
      />
    {/if}

    <Field
      label="Priority"
      value={{ integer: priority }}
      onchange={(v: Value) =>
        (priority = "integer" in v ? v.integer : priority)}
    />

    <div class="actions">
      <button type="submit" class="btn-brutalist create">
        {page ? "Save" : "Create"}
      </button>
    </div>
  </form>
</dialog>

<style>
  .dialog {
    width: min(320px, 100%);
    padding: 20px;
    border: 2px solid var(--accent);
    border-radius: var(--radius);
    background: var(--surface-raised);
    color: var(--text);
    box-shadow: 6px 6px 0 var(--accent-shadow);
  }

  .dialog::backdrop {
    background: rgba(0, 0, 0, 0.5);
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
