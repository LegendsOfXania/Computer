<script lang="ts">
  import { ChevronDown, ChevronRight, Plus, X } from "lucide-svelte";
  import Field from "./Field.svelte";
  import type { Value } from "$lib/types/model";

  let {
    label,
    value,
    onchange,
  }: { label: string; value: Value; onchange: (value: Value) => void } =
    $props();

  let expanded = $state(false);

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

  function scalarClass(value: Value): string {
    if (typeof value !== "object" || value === null) return "";
    if ("reference" in value) return "reference";
    if ("enum" in value) return "enum";
    return "";
  }

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

  function blankLike(template: Value): Value {
    if (typeof template !== "object" || template === null) return { text: "" };
    if ("text" in template) return { text: "" };
    if ("enum" in template) return { enum: "" };
    if ("reference" in template) return { reference: "" };
    if ("integer" in template) return { integer: 0 };
    if ("float" in template) return { float: 0 };
    if ("boolean" in template) return { boolean: false };
    if ("list" in template) return { list: [] };
    if ("struct" in template) {
      return {
        struct: Object.fromEntries(
          Object.entries(template.struct).map(([key, nested]) => [
            key,
            blankLike(nested),
          ]),
        ),
      };
    }
    return "null";
  }

  function addListItem() {
    if (typeof value !== "object" || value === null || !("list" in value))
      return;
    onchange({
      list: [...value.list, blankLike(value.list[0] ?? { text: "" })],
    });
    expanded = true;
  }

  function removeListItem(index: number) {
    if (typeof value !== "object" || value === null || !("list" in value))
      return;
    onchange({ list: value.list.filter((_, i) => i !== index) });
  }
</script>

{#if typeof value === "object" && value !== null && "list" in value}
  <div class="field">
    <div class="field-box list-header">
      <button
        type="button"
        class="list-toggle"
        onclick={() => (expanded = !expanded)}
      >
        {#if expanded}
          <ChevronDown size={14} />
        {:else}
          <ChevronRight size={14} />
        {/if}
        <span>{label}</span>
        <span class="list-count">({value.list.length})</span>
      </button>
      <button
        type="button"
        class="list-add"
        onclick={addListItem}
        title="Add item"
      >
        <Plus size={14} />
      </button>
    </div>

    <div
      class="list-content-wrapper {expanded && value.list.length > 0
        ? 'expanded'
        : ''}"
    >
      <div class="list-content-inner">
        {#if value.list.length > 0}
          <div class="field-nested">
            {#each value.list as item, index (index)}
              <div class="list-item">
                <Field
                  label={String(index)}
                  value={item}
                  onchange={(v) => handleListItemChange(index, v)}
                />
                <button
                  type="button"
                  class="list-remove"
                  onclick={() => removeListItem(index)}
                  title="Remove item"
                >
                  <X size={14} />
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
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
      class="field-box {scalarClass(value)}"
      type={inputType(value)}
      step={isFloat(value) ? "any" : undefined}
      value={formatScalar(value)}
      oninput={handleScalarInput}
    />
  </div>
{/if}
