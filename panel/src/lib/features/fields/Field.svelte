<script lang="ts">
  import type { Schema, Value } from "$lib/domain/model";
  import BooleanField from "./BooleanField.svelte";
  import EnumField from "./EnumField.svelte";
  import ListField from "./ListField.svelte";
  import ReferenceField from "./ReferenceField.svelte";
  import ScalarField from "./ScalarField.svelte";
  import StructField from "./StructField.svelte";
  import "./fields.css";

  let {
    label = "",
    value,
    onchange,
    schema,
  }: {
    label?: string;
    value: Value;
    onchange: (value: Value) => void;
    schema?: Schema;
  } = $props();
</script>

{#if "list" in value}
  <ListField {label} {value} {schema} {onchange} />
{:else if "struct" in value}
  <StructField {label} {value} {schema} {onchange} />
{:else if "reference" in value}
  <ReferenceField {label} {value} {schema} {onchange} />
{:else if "boolean" in value}
  <BooleanField {label} {value} {onchange} />
{:else if "enum" in value}
  <EnumField {label} {value} {schema} {onchange} />
{:else}
  <ScalarField {label} {value} {onchange} />
{/if}
