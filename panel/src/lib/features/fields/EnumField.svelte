<script lang="ts">
  import type { Schema, Value } from "$lib/domain/model";
  let {
    label = "",
    value,
    schema,
    onchange,
  }: {
    label?: string;
    value: { enum: string };
    schema?: Schema;
    onchange: (value: Value) => void;
  } = $props();
  const options = $derived(
    schema && typeof schema === "object" && "enumeration" in schema
      ? schema.enumeration
      : [],
  );
</script>

<div class="field">
  {#if label}<span class="field-label">{label}</span>{/if}
  {#if options.length}
    <select
      class="field-box"
      value={value.enum}
      onchange={(event) => onchange({ enum: event.currentTarget.value })}
    >
      {#each options as option}<option value={option}>{option}</option>{/each}
    </select>
  {:else}
    <input
      class="field-box"
      value={value.enum}
      oninput={(event) => onchange({ enum: event.currentTarget.value })}
    />
  {/if}
</div>
