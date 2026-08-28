<script lang="ts">
  import { ChevronDown, ChevronRight, Plus, X } from "lucide-svelte";
  import Field from "./Field.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { displayName } from "$lib/types/model";
  import type { Value } from "$lib/types/model";

  let {
    label = "",
    value,
    onchange,
  }: {
    label?: string;
    value: Value;
    onchange: (value: Value) => void;
  } = $props();

  let expanded = $state(false);

  function formatScalar(value: Value): string {
    if (value === "null") return "";
    if (typeof value !== "object" || value === null) return "";
    if ("text" in value) return value.text;
    if ("enum" in value) return value.enum;
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
  <div class="field list-field">
    <div class="list-header">
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
    {#if label}<span class="field-label">{label}</span>{/if}
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
{:else if typeof value === "object" && value !== null && "reference" in value}
  {@const target = value.reference ? appStore.findEntry(value.reference) : null}
  <div class="field">
    {#if label}<span class="field-label">{label}</span>{/if}
    <div class="reference-preview" class:broken={value.reference && !target}>
      {#if target}
        <span class="reference-dot"></span>
        <span class="reference-type">{target.entry_type}</span>
        <span class="reference-name">{displayName(target)}</span>
      {:else if value.reference}
        <span class="reference-dot broken"></span>
        <span class="reference-name muted">Introuvable</span>
        <span class="reference-id">{value.reference}</span>
      {:else}
        <span class="reference-name muted">Aucune référence</span>
      {/if}
    </div>
  </div>
{:else if typeof value === "object" && value !== null && "boolean" in value}
  <div class="field">
    {#if label}<span class="field-label">{label}</span>{/if}
    <label class="field-box toggle">
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
    {#if label}<span class="field-label">{label}</span>{/if}
    <input class="field-box" type="text" placeholder="—" disabled />
  </div>
{:else}
  <div class="field">
    {#if label}<span class="field-label">{label}</span>{/if}
    <input
      class="field-box {scalarClass(value)}"
      type={inputType(value)}
      step={isFloat(value) ? "any" : undefined}
      value={formatScalar(value)}
      oninput={handleScalarInput}
    />
  </div>
{/if}

<style>
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    width: 100%;
  }

  .field-label {
    display: flex;
    align-items: center;
    gap: 6px;

    font-family: ui-monospace, "SF Mono", "Cascadia Code", Menlo, Consolas,
      monospace;
    font-size: 10.5px;
    font-weight: 600;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  .field-label::before {
    content: "";
    width: 5px;
    height: 5px;
    flex-shrink: 0;
    background: var(--accent);
    opacity: 0.6;
  }

  .field-box {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;

    min-height: 36px;
    padding: 0 10px;

    border: 1px solid var(--border-muted);
    border-radius: var(--radius);

    background: var(--surface-raised);
    color: var(--text);

    font-size: 13px;
    font-family: inherit;

    transition:
      border-color 0.15s ease,
      box-shadow 0.15s ease,
      transform 0.1s ease;
  }

  label.field-box {
    cursor: pointer;
  }

  .field-box:hover:not(:disabled) {
    border-color: var(--text-muted);
  }

  .field-box:focus,
  .field-box:focus-within {
    border-color: var(--accent);
    outline: none;
    box-shadow: 2px 2px 0 var(--accent-shadow);
  }

  .field-box::placeholder {
    color: var(--text-muted);
  }

  .field-box:disabled {
    color: var(--text-muted);
    border-style: dashed;
    cursor: not-allowed;
  }

  .field-box.reference,
  .field-box.enum {
    font-family: ui-monospace, "SF Mono", "Cascadia Code", Menlo, Consolas,
      monospace;
  }

  .field-box.empty {
    color: var(--text-muted);
    font-style: italic;
  }

  .field-box[type="number"] {
    appearance: textfield;
    -moz-appearance: textfield;
  }

  .field-box[type="number"]::-webkit-outer-spin-button,
  .field-box[type="number"]::-webkit-inner-spin-button {
    margin: 0;
    -webkit-appearance: none;
  }

  .field-box.toggle {
    gap: 10px;
  }

  .field-box.toggle input[type="checkbox"] {
    appearance: none;
    position: relative;
    width: 34px;
    height: 20px;
    margin: 0;
    flex-shrink: 0;
    border-radius: 999px;
    background: var(--border-muted);
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .field-box.toggle input[type="checkbox"]::before {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--text-muted);
    transition:
      transform 0.15s ease,
      background-color 0.15s ease;
  }

  .field-box.toggle input[type="checkbox"]:checked {
    background: var(--accent);
  }

  .field-box.toggle input[type="checkbox"]:checked::before {
    transform: translateX(14px);
    background: var(--on-accent);
  }

  .field-box.toggle input[type="checkbox"]:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .field-box.toggle span {
    font-family: ui-monospace, "SF Mono", "Cascadia Code", Menlo, Consolas,
      monospace;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  .reference-preview {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 36px;
    padding: 0 10px;

    border: 1px solid var(--border-muted);
    border-radius: var(--radius);
    background: var(--surface-raised);
  }

  .reference-dot {
    width: 6px;
    height: 6px;
    flex-shrink: 0;
    border-radius: 50%;
    background: var(--accent);
  }

  .reference-dot.broken {
    background: var(--warning);
  }

  .reference-type {
    flex-shrink: 0;
    font-family: ui-monospace, "SF Mono", "Cascadia Code", Menlo, Consolas,
      monospace;
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--accent);
  }

  .reference-name {
    overflow: hidden;
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .reference-name.muted {
    font-weight: 400;
    font-style: italic;
    color: var(--text-muted);
  }

  .reference-id {
    flex-shrink: 0;
    margin-left: auto;
    font-family: ui-monospace, "SF Mono", "Cascadia Code", Menlo, Consolas,
      monospace;
    font-size: 11px;
    color: var(--text-muted);
  }

  .reference-preview.broken {
    border-style: dashed;
    border-color: var(--warning);
    background: transparent;
  }

  .field-nested {
    display: flex;
    flex-direction: column;
    gap: 12px;

    padding: 2px 0 2px 14px;
    border-left: 2px solid var(--border-muted);
  }

  .list-field {
    width: 100%;
  }

  .list-content-wrapper {
    display: grid;
    grid-template-rows: 0fr;
    opacity: 0;
    transition:
      grid-template-rows 0.25s cubic-bezier(0.4, 0, 0.2, 1),
      opacity 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .list-content-wrapper.expanded {
    grid-template-rows: 1fr;
    opacity: 1;
  }

  .list-content-inner {
    overflow: hidden;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding-top: 8px;
  }

  .list-field .list-header {
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border-muted);
  }

  .list-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .list-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    padding: 2px 0;

    border: none;
    background: none;
    color: var(--text);

    font-family: ui-monospace, "SF Mono", "Cascadia Code", Menlo, Consolas,
      monospace;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    cursor: pointer;

    transition: color 0.15s ease;
  }

  .list-toggle:hover {
    color: var(--accent);
  }

  .list-toggle :global(svg) {
    flex-shrink: 0;
    color: var(--text-muted);
    transition: color 0.15s ease;
  }

  .list-toggle:hover :global(svg) {
    color: var(--accent);
  }

  .list-count {
    font-weight: 400;
    letter-spacing: 0;
    text-transform: none;
    color: var(--text-muted);
  }

  .list-add {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;

    width: 22px;
    height: 22px;

    border: 1px solid var(--border-muted);
    border-radius: 3px;
    background: none;
    color: var(--text-muted);
    cursor: pointer;

    transition:
      color 0.15s ease,
      background-color 0.15s ease,
      border-color 0.15s ease;
  }

  .list-add:hover {
    color: var(--on-accent);
    background: var(--accent);
    border-color: var(--accent);
  }

  .list-item {
    display: flex;
    align-items: flex-start;
    gap: 6px;
    width: 100%;
  }

  .list-remove {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;

    width: 28px;
    height: 36px;

    border: none;
    background: none;
    color: var(--text-muted);
    cursor: pointer;
    opacity: 0.5;

    transition:
      color 0.15s ease,
      opacity 0.15s ease;
  }

  .list-item:hover .list-remove {
    opacity: 1;
  }

  .list-remove:hover {
    color: var(--danger);
    opacity: 1;
  }
</style>
