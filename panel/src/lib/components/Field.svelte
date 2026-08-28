<script lang="ts">
  import { ChevronDown, ChevronRight, Plus, X } from "lucide-svelte";
  import Field from "./Field.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { displayName, blankLike, type Value } from "$lib/types/model";

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

  const scalar = (v: Value) =>
    v === "null"
      ? ""
      : typeof v === "object" && v
        ? "text" in v
          ? v.text
          : "enum" in v
            ? v.enum
            : "integer" in v
              ? String(v.integer)
              : "float" in v
                ? String(v.float)
                : ""
        : "";
  const numeric = (v: Value) =>
    typeof v === "object" && v && ("integer" in v || "float" in v);
  function update(raw: string) {
    if (typeof value === "object" && value) {
      if ("integer" in value) {
        const n = Number.parseInt(raw, 10);
        onchange({ integer: Number.isNaN(n) ? 0 : n });
        return;
      }
      if ("float" in value) {
        const n = Number.parseFloat(raw);
        onchange({ float: Number.isNaN(n) ? 0 : n });
        return;
      }
      if ("enum" in value) {
        onchange({ enum: raw });
        return;
      }
    }
    onchange({ text: raw });
  }
  function updateList(i: number, v: Value) {
    if (typeof value === "object" && value && "list" in value)
      onchange({ list: value.list.map((x, n) => (n === i ? v : x)) });
  }
  function updateStruct(k: string, v: Value) {
    if (typeof value === "object" && value && "struct" in value)
      onchange({ struct: { ...value.struct, [k]: v } });
  }
  function add() {
    if (typeof value === "object" && value && "list" in value) {
      onchange({
        list: [...value.list, blankLike(value.list[0] ?? { text: "" })],
      });
      expanded = true;
    }
  }
  function remove(i: number) {
    if (typeof value === "object" && value && "list" in value)
      onchange({ list: value.list.filter((_, n) => n !== i) });
  }
</script>

{#if typeof value === "object" && value && "list" in value}
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
      <button type="button" class="list-add" onclick={add} title="Add item"
        ><Plus size={14} /></button
      >
    </div>
    {#if expanded && value.list.length}
      <div class="nested">
        {#each value.list as item, i (i)}
          <div class="list-item">
            <Field value={item} onchange={(v) => updateList(i, v)} /><button
              type="button"
              class="remove"
              onclick={() => remove(i)}
              title="Remove item"><X size={14} /></button
            >
          </div>
        {/each}
      </div>
    {/if}
  </div>
{:else if typeof value === "object" && value && "struct" in value}
  <div class="field">
    {#if label}<span class="field-label">{label}</span>{/if}
    <div class="nested">
      {#each Object.entries(value.struct) as [key, nested] (key)}<Field
          label={key}
          value={nested}
          onchange={(v) => updateStruct(key, v)}
        />{/each}
    </div>
  </div>
{:else if typeof value === "object" && value && "reference" in value}
  {@const target = value.reference
    ? appStore.findEntry(value.reference)
    : undefined}
  <div class="field">
    {#if label}<span class="field-label">{label}</span>{/if}
    <div class:broken={!!value.reference && !target} class="reference">
      {#if target}<span class="dot"></span><span class="type"
          >{target.entry_type}</span
        ><span class="name">{displayName(target)}</span>{:else}<span
          class="name muted"
          >{value.reference ? "Introuvable" : "Aucune référence"}</span
        >{#if value.reference}<span class="id">{value.reference}</span
          >{/if}{/if}
    </div>
  </div>
{:else if typeof value === "object" && value && "boolean" in value}
  <div class="field">
    {#if label}<span class="field-label">{label}</span>{/if}<label
      class="field-box toggle"
      ><input
        type="checkbox"
        checked={value.boolean}
        onchange={(e) =>
          onchange({ boolean: (e.currentTarget as HTMLInputElement).checked })}
      /><span>{value.boolean ? "True" : "False"}</span></label
    >
  </div>
{:else}
  <div class="field">
    {#if label}<span class="field-label">{label}</span>{/if}<input
      class="field-box"
      type={numeric(value) ? "number" : "text"}
      step={typeof value === "object" && value && "float" in value
        ? "any"
        : undefined}
      value={scalar(value)}
      disabled={value === "null"}
      placeholder="—"
      oninput={(e) => update((e.currentTarget as HTMLInputElement).value)}
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
  .field-label,
  .list-toggle {
    font-family: ui-monospace, monospace;
    font-size: 10.5px;
    font-weight: 600;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: var(--text-muted);
  }
  .field-label {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .field-label::before {
    content: "";
    width: 5px;
    height: 5px;
    background: var(--accent);
    opacity: 0.6;
  }
  .field-box,
  .reference {
    width: 100%;
    min-height: 36px;
    padding: 0 10px;
    border: 1px solid var(--border-muted);
    border-radius: var(--radius);
    background: var(--surface-raised);
    color: var(--text);
    font-size: 13px;
  }
  .field-box:focus,
  .field-box:focus-within {
    border-color: var(--accent);
    outline: 0;
    box-shadow: 2px 2px 0 var(--accent-shadow);
  }
  .field-box:disabled {
    color: var(--text-muted);
    border-style: dashed;
  }
  .field-box[type="number"] {
    appearance: textfield;
  }
  .toggle {
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
  }
  .toggle input {
    width: 34px;
    height: 20px;
    margin: 0;
    accent-color: var(--accent);
  }
  .nested {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 2px 0 2px 14px;
    border-left: 2px solid var(--border-muted);
  }
  .list-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border-muted);
  }
  .list-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    padding: 2px 0;
    border: 0;
    background: none;
    color: var(--text);
    cursor: pointer;
  }
  .list-toggle:hover {
    color: var(--accent);
  }
  .muted {
    color: var(--text-muted);
    font-weight: 400;
  }
  .list-add {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border: 1px solid var(--border-muted);
    border-radius: 3px;
    background: none;
    color: var(--text-muted);
    cursor: pointer;
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
  .remove {
    width: 28px;
    height: 36px;
    border: 0;
    background: none;
    color: var(--text-muted);
    opacity: 0.5;
    cursor: pointer;
  }
  .list-item:hover .remove {
    opacity: 1;
  }
  .remove:hover {
    color: var(--danger);
  }
  .reference {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    flex: 0 0 auto;
  }
  .type {
    font-family: ui-monospace, monospace;
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--accent);
  }
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
    font-weight: 600;
  }
  .id {
    margin-left: auto;
    font-family: ui-monospace, monospace;
    font-size: 11px;
    color: var(--text-muted);
  }
  .reference.broken {
    border-style: dashed;
    border-color: var(--warning);
    background: transparent;
  }
</style>
