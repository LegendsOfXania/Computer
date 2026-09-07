<script lang="ts">
  import type { Schema, Value } from "$lib/domain/model";
  import Field from "$lib/features/fields/Field.svelte";
  let {
    label = "",
    value,
    schema,
    onchange,
  }: {
    label?: string;
    value: { struct: Record<string, Value> };
    schema?: Schema;
    onchange: (value: Value) => void;
  } = $props();
  function fieldSchema(key: string) {
    return schema && typeof schema === "object" && "struct" in schema
      ? schema.struct.find((field) => field.name === key)?.schema
      : undefined;
  }
  function update(key: string, next: Value) {
    onchange({ struct: { ...value.struct, [key]: next } });
  }
</script>

<div class="field">
  {#if label}<span class="field-label">{label}</span>{/if}
  <div class="nested">
    {#each Object.entries(value.struct) as [key, nested] (key)}
      <Field
        label={key}
        value={nested}
        schema={fieldSchema(key)}
        onchange={(next) => update(key, next)}
      />
    {/each}
  </div>
</div>
