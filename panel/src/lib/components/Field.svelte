<script lang="ts">
  import Field from "./Field.svelte";
  import type { Value } from "$lib/types/model";

  let {
    label,
    value,
    onchange,
  }: { label: string; value: Value; onchange: (value: Value) => void } =
    $props();

  function formatScalar(value: Value): string {
    if (value === "null") return "";
    if (typeof value !== "object" || value === null) return "";
    if ("text" in value) return value.text;
    if ("enum" in value) return value.enum;
    if ("reference" in value) return value.reference;
    if ("integer" in value) return String(value.integer);
    if ("float" in value) return String(value.float);
    return "";
  }

  function isFloat(value: Value): boolean {
    return typeof value === "object" && value !== null && "float" in value;
  }

  function inputType(value: Value): "number" | "text" {
    if (typeof value !== "object" || value === null) return "text";
    return "integer" in value || "float" in value ? "number" : "text";
  }

  // Turns the input's raw string back into the same Value shape it started
  // as, so e.g. editing an integer field keeps producing `{ integer }`.
  function scalarFromInput(previous: Value, raw: string): Value {
    if (typeof previous === "object" && previous !== null) {
      if ("integer" in previous) {
        const parsed = Number.parseInt(raw, 10);
        return { integer: Number.isNaN(parsed) ? 0 : parsed };
      }
      if ("float" in previous) {
        const parsed = Number.parseFloat(raw);
        return { float: Number.isNaN(parsed) ? 0 : parsed };
      }
      if ("enum" in previous) return { enum: raw };
      if ("reference" in previous) return { reference: raw };
    }
    return { text: raw };
  }

  function handleScalarInput(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    onchange(scalarFromInput(value, target.value));
  }

  function handleBooleanChange(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    onchange({ boolean: target.checked });
  }

  function handleListItemChange(index: number, newItem: Value) {
    if (typeof value !== "object" || value === null || !("list" in value))
      return;
    onchange({
      list: value.list.map((item, i) => (i === index ? newItem : item)),
    });
  }

  function handleStructFieldChange(key: string, newField: Value) {
    if (typeof value !== "object" || value === null || !("struct" in value))
      return;
    onchange({ struct: { ...value.struct, [key]: newField } });
  }
</script>

{#if typeof value === "object" && value !== null && "list" in value}
  <div class="field">
    <span class="field-label">{label}</span>
    {#if value.list.length === 0}
      <div class="field-box">Empty list</div>
    {:else}
      <div class="field-nested">
        {#each value.list as item, index (index)}
          <Field
            label={String(index)}
            value={item}
            onchange={(v) => handleListItemChange(index, v)}
          />
        {/each}
      </div>
    {/if}
  </div>
{:else if typeof value === "object" && value !== null && "struct" in value}
  <div class="field">
    <span class="field-label">{label}</span>
    <div class="field-nested">
      {#each Object.entries(value.struct) as [key, nested] (key)}
        <Field
          label={key}
          value={nested}
          onchange={(v) => handleStructFieldChange(key, v)}
        />
      {/each}
    </div>
  </div>
{:else if typeof value === "object" && value !== null && "boolean" in value}
  <div class="field">
    <span class="field-label">{label}</span>
    <label class="field-box">
      <input
        type="checkbox"
        checked={value.boolean}
        onchange={handleBooleanChange}
      />
      <span>{value.boolean ? "True" : "False"}</span>
    </label>
  </div>
{:else if value === "null"}
  <div class="field">
    <span class="field-label">{label}</span>
    <input class="field-box" type="text" placeholder="—" disabled />
  </div>
{:else}
  <div class="field">
    <span class="field-label">{label}</span>
    <input
      class="field-box"
      type={inputType(value)}
      step={isFloat(value) ? "any" : undefined}
      value={formatScalar(value)}
      oninput={handleScalarInput}
    />
  </div>
{/if}
