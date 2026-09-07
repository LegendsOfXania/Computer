<script lang="ts">
  import { Search } from "lucide-svelte";
  import { appStore } from "$lib/stores/app.svelte";
  import { displayName, type Schema, type Value } from "$lib/domain/model";
  import SearchDialog from "$lib/ui/SearchDialog.svelte";
  let {
    label = "",
    value,
    schema,
    onchange,
  }: {
    label?: string;
    value: { reference: string };
    schema?: Schema;
    onchange: (value: Value) => void;
  } = $props();
  let showPicker = $state(false);
  const reference = $derived(value.reference);
  const target = $derived(
    reference ? appStore.getEntryData(reference) : undefined,
  );
  const referenceSchema = $derived(
    schema && typeof schema === "object" && "reference" in schema
      ? schema.reference
      : undefined,
  );
  const pickerQuery = $derived(
    referenceSchema?.tags?.length
      ? `!entry !tags:${referenceSchema.tags.join(",")}`
      : "!entry",
  );
  $effect(() => {
    if (reference && !target) appStore.requestEntry(reference);
  });
</script>

<div class="field">
  {#if label}<span class="field-label">{label}</span>{/if}
  <div class="reference-row">
    <button
      type="button"
      class="reference"
      class:broken={!!reference && !target}
      disabled={!reference}
      onclick={() => reference && appStore.openReference(reference)}
    >
      {#if target}<span class="dot"></span><span class="type"
          >{target.entry_type}</span
        ><span class="name">{displayName(target)}</span>
      {:else if reference}<span class="name muted">Chargement...</span>
      {:else}<span class="name muted">Aucune référence</span>{/if}
    </button>
    <button
      type="button"
      class="reference-pick"
      onclick={() => (showPicker = true)}
      aria-label={reference ? "Changer la référence" : "Choisir une référence"}
      ><Search size={14} /></button
    >
  </div>
</div>
<SearchDialog
  bind:open={showPicker}
  fixedQuery={pickerQuery}
  onSelectEntry={(key) => onchange({ reference: key })}
/>

//todo when creating a new ref, with a new entry, automaticaly connect them
