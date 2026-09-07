<script lang="ts">
  import type { Value } from "$lib/domain/model";
  let {
    label = "",
    value,
    onchange,
  }: {
    label?: string;
    value: { boolean: boolean };
    onchange: (value: Value) => void;
  } = $props();
</script>

<div class="field">
  {#if label}<span class="field-label">{label}</span>{/if}
  <label class="field-box toggle">
    <input
      type="checkbox"
      checked={value.boolean}
      onchange={(event) => onchange({ boolean: event.currentTarget.checked })}
    />
    <span>{value.boolean ? "True" : "False"}</span>
  </label>
</div>
