<script lang="ts">
  import Field from "./Field.svelte";
  import type { Value } from "$lib/types/model";

  let { label, value }: { label: string; value: Value } = $props();

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
</script>

{#if typeof value === "object" && value !== null && "list" in value}
  <div class="field">
    <span class="field-label">{label}</span>
    {#if value.list.length === 0}
      <div class="field-box">Empty list</div>
    {:else}
      <div class="field-nested">
        {#each value.list as item, index (index)}
          <Field label={String(index)} value={item} />
        {/each}
      </div>
    {/if}
  </div>
{:else if typeof value === "object" && value !== null && "struct" in value}
  <div class="field">
    <span class="field-label">{label}</span>
    <div class="field-nested">
      {#each Object.entries(value.struct) as [key, nested] (key)}
        <Field label={key} value={nested} />
      {/each}
    </div>
  </div>
{:else if typeof value === "object" && value !== null && "boolean" in value}
  <div class="field">
    <span class="field-label">{label}</span>
    <label class="field-box">
      <input type="checkbox" checked={value.boolean} disabled />
      <span>{value.boolean ? "True" : "False"}</span>
    </label>
  </div>
{:else}
  <div class="field">
    <span class="field-label">{label}</span>
    <input class="field-box" type="text" readonly value={formatScalar(value)} />
  </div>
{/if}
