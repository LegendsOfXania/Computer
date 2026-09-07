<script lang="ts">
  import { X } from "lucide-svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import {
    PAGE_TYPES,
    type PageInfo,
    type PageType,
    type Schema,
    type Value,
  } from "$lib/domain/model";
  import Field from "$lib/features/fields/Field.svelte";
  import Dialog from "./Dialog.svelte";

  let {
    open = $bindable(false),
    page = null,
  }: { open?: boolean; page?: PageInfo | null } = $props();
  const pageTypeSchema: Schema = { enumeration: PAGE_TYPES };
  let name = $state("");
  let pageType = $state<PageType>(PAGE_TYPES[0]);
  let priority = $state(0);

  function reset() {
    name = page?.name ?? "New page";
    pageType = page?.page_type ?? PAGE_TYPES[0];
    priority = page?.priority ?? 0;
  }
  $effect(() => {
    if (open) reset();
  });
  function submit(event: SubmitEvent) {
    event.preventDefault();
    const trimmed = name.trim();
    if (!trimmed) return;
    if (page) appStore.editPage(page.id, trimmed, priority);
    else appStore.createPage(trimmed, pageType, priority);
    open = false;
  }
</script>

<Dialog bind:open>
  <div class="header">
    <h2>{page ? "Edit page" : "New page"}</h2>
    <button
      type="button"
      class="close"
      aria-label="Close"
      onclick={() => (open = false)}><X size={16} /></button
    >
  </div>
  <form onsubmit={submit}>
    <Field
      label="Name"
      value={{ text: name }}
      onchange={(value: Value) => "text" in value && (name = value.text)}
    />
    {#if !page}<Field
        label="Type"
        value={{ enum: pageType }}
        schema={pageTypeSchema}
        onchange={(value: Value) =>
          "enum" in value && (pageType = value.enum as PageType)}
      />{/if}
    <Field
      label="Priority"
      value={{ integer: priority }}
      onchange={(value: Value) =>
        "integer" in value && (priority = value.integer)}
    />
    <div class="actions">
      <button type="button" onclick={() => (open = false)}>Cancel</button
      ><button type="submit" class="btn-brutalist" data-autofocus
        >{page ? "Save" : "Create"}</button
      >
    </div>
  </form>
</Dialog>
