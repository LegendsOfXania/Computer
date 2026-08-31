<script lang="ts">
  import { ChevronDown, ChevronRight, Plus, X } from "lucide-svelte";
  import Field from "./Field.svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { defaultValue, displayName, type Value } from "$lib/types/model";

  let {
    label = "",
    value,
    onchange,
    options = [],
  }: {
    label?: string;
    value: Value;
    onchange: (value: Value) => void;
    options?: string[];
  } = $props();

  let expanded = $state(false);

  const reference = $derived(
    typeof value === "object" && value && "reference" in value
      ? value.reference
      : undefined,
  );

  const target = $derived(
    reference ? appStore.getEntryData(reference) : undefined,
  );

  $effect(() => {
    if (reference && !target) {
      appStore.requestEntry(reference);
    }
  });

  function scalar(value: Value): string {
    if (value === "null") {
      return "";
    }

    if (typeof value !== "object" || value === null) {
      return "";
    }

    if ("text" in value) {
      return value.text;
    }

    if ("enum" in value) {
      return value.enum;
    }

    if ("integer" in value) {
      return String(value.integer);
    }

    if ("float" in value) {
      return String(value.float);
    }

    return "";
  }

  function numeric(value: Value): boolean {
    return (
      typeof value === "object" &&
      value !== null &&
      ("integer" in value || "float" in value)
    );
  }

  function update(raw: string) {
    if (typeof value === "object" && value !== null) {
      if ("integer" in value) {
        const number = Number.parseInt(raw, 10);

        onchange({
          integer: Number.isNaN(number) ? 0 : number,
        });

        return;
      }

      if ("float" in value) {
        const number = Number.parseFloat(raw);

        onchange({
          float: Number.isNaN(number) ? 0 : number,
        });

        return;
      }

      if ("enum" in value) {
        onchange({
          enum: raw,
        });

        return;
      }
    }

    onchange({
      text: raw,
    });
  }

  function updateList(index: number, next: Value) {
    if (typeof value === "object" && value !== null && "list" in value) {
      onchange({
        list: value.list.map((item, i) => (i === index ? next : item)),
      });
    }
  }

  function updateStruct(key: string, next: Value) {
    if (typeof value === "object" && value !== null && "struct" in value) {
      onchange({
        struct: {
          ...value.struct,
          [key]: next,
        },
      });
    }
  }

  function add() {
    if (typeof value === "object" && value !== null && "list" in value) {
      onchange({
        list: [...value.list, defaultValue(value.list[0] ?? { text: "" })],
      });

      expanded = true;
    }
  }

  function remove(index: number) {
    if (typeof value === "object" && value !== null && "list" in value) {
      onchange({
        list: value.list.filter((_, i) => i !== index),
      });
    }
  }
</script>

{#if typeof value === "object" && value !== null && "list" in value}
  <div class="field">
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
        <span class="muted">({value.list.length})</span>
      </button>

      <button
        type="button"
        class="list-add"
        onclick={add}
        title="Add item"
        aria-label="Add item"
      >
        <Plus size={14} />
      </button>
    </div>

    {#if expanded && value.list.length > 0}
      <div class="nested">
        {#each value.list as item, i (i)}
          <div class="list-item">
            <Field value={item} onchange={(next) => updateList(i, next)} />

            <button
              type="button"
              class="remove"
              onclick={() => remove(i)}
              title="Remove item"
              aria-label="Remove item"
            >
              <X size={14} />
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </div>
{:else if typeof value === "object" && value !== null && "struct" in value}
  <div class="field">
    {#if label}
      <span class="field-label">{label}</span>
    {/if}

    <div class="nested">
      {#each Object.entries(value.struct) as [key, nested] (key)}
        <Field
          label={key}
          value={nested}
          onchange={(next) => updateStruct(key, next)}
        />
      {/each}
    </div>
  </div>
{:else if reference !== undefined}
  <div class="field">
    {#if label}
      <span class="field-label">{label}</span>
    {/if}

    <button
      type="button"
      class="reference"
      class:broken={!!reference && !target}
      disabled={!reference}
      onclick={() => reference && appStore.openReference(reference)}
    >
      {#if target}
        <span class="dot"></span>

        <span class="type">
          {target.entry_type}
        </span>

        <span class="name">
          {displayName(target)}
        </span>
      {:else if reference}
        <span class="name muted">Chargement...</span>
      {:else}
        <span class="name muted">Aucune référence</span>
      {/if}
    </button>
  </div>
{:else if typeof value === "object" && value !== null && "boolean" in value}
  <div class="field">
    {#if label}
      <span class="field-label">{label}</span>
    {/if}

    <label class="field-box toggle">
      <input
        type="checkbox"
        checked={value.boolean}
        onchange={(event) => {
          onchange({
            boolean: event.currentTarget.checked,
          });
        }}
      />

      <span>{value.boolean ? "True" : "False"}</span>
    </label>
  </div>
{:else if typeof value === "object" && value !== null && "enum" in value && options.length > 0}
  <div class="field">
    {#if label}
      <span class="field-label">{label}</span>
    {/if}

    <select
      class="field-box"
      value={value.enum}
      onchange={(event) => {
        onchange({
          enum: event.currentTarget.value,
        });
      }}
    >
      {#each options as option}
        <option value={option}>{option}</option>
      {/each}
    </select>
  </div>
{:else}
  <div class="field">
    {#if label}
      <span class="field-label">{label}</span>
    {/if}

    <input
      class="field-box"
      type={numeric(value) ? "number" : "text"}
      step={typeof value === "object" && value !== null && "float" in value
        ? "any"
        : undefined}
      value={scalar(value)}
      disabled={value === "null"}
      placeholder="—"
      oninput={(event) => {
        update(event.currentTarget.value);
      }}
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
    box-sizing: border-box;
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
    user-select: none;
  }

  .toggle input {
    appearance: none;
    position: relative;
    width: 34px;
    height: 20px;
    margin: 0;
    flex: 0 0 auto;
    border: 1px solid var(--border-muted);
    border-radius: 999px;
    background: var(--surface);
    cursor: pointer;
    transition:
      background 120ms ease,
      border-color 120ms ease;
  }

  .toggle input::before {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--text-muted);
    transition:
      transform 120ms ease,
      background 120ms ease;
  }

  .toggle input:checked {
    background: var(--accent);
    border-color: var(--accent);
  }

  .toggle input:checked::before {
    transform: translateX(14px);
    background: var(--on-accent);
  }

  .toggle input:focus-visible {
    outline: 0;
    box-shadow: 2px 2px 0 var(--accent-shadow);
  }

  .toggle span {
    font-size: 13px;
    color: var(--text);
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
    font-family: inherit;
    text-align: left;
    cursor: pointer;
  }

  .reference:disabled {
    cursor: default;
  }

  .reference:not(:disabled):hover {
    border-color: var(--accent);
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

  .reference.broken {
    border-style: dashed;
    border-color: var(--warning);
    background: transparent;
  }
</style>
