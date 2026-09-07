<script lang="ts">
  import { ChevronDown, ChevronRight, Plus, X } from "lucide-svelte";
  import {
    defaultSchema,
    defaultValue,
    type Schema,
    type Value,
  } from "$lib/domain/model";
  import Field from "$lib/features/fields/Field.svelte";
  let {
    label = "",
    value,
    schema,
    onchange,
  }: {
    label?: string;
    value: { list: Value[] };
    schema?: Schema;
    onchange: (value: Value) => void;
  } = $props();
  let expanded = $state(false);
  const elementSchema = $derived(
    schema && typeof schema === "object" && "list" in schema
      ? schema.list
      : undefined,
  );
  function add() {
    const next = elementSchema
      ? defaultSchema(elementSchema)
      : defaultValue(value.list[0] ?? { text: "" });
    onchange({ list: [...value.list, next] });
    expanded = true;
  }
  function remove(index: number) {
    onchange({ list: value.list.filter((_, i) => i !== index) });
  }
  function update(index: number, next: Value) {
    onchange({
      list: value.list.map((item, i) => (i === index ? next : item)),
    });
  }
</script>

<div class="field">
  <div class="list-header">
    <button
      type="button"
      class="list-toggle"
      onclick={() => (expanded = !expanded)}
    >
      {#if expanded}<ChevronDown size={14} />{:else}<ChevronRight
          size={14}
        />{/if}
      <span>{label}</span><span class="muted">({value.list.length})</span>
    </button>
    <button
      type="button"
      class="list-add"
      onclick={add}
      aria-label="Add item"
      title="Add item"><Plus size={14} /></button
    >
  </div>
  {#if expanded && value.list.length}
    <div class="nested">
      {#each value.list as item, index (index)}
        <div class="list-item">
          <Field
            value={item}
            schema={elementSchema}
            onchange={(next) => update(index, next)}
          />
          <button
            type="button"
            class="remove"
            onclick={() => remove(index)}
            aria-label="Remove item"
            title="Remove item"><X size={14} /></button
          >
        </div>
      {/each}
    </div>
  {/if}
</div>
