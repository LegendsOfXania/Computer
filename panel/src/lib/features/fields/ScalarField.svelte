<script lang="ts">
  import type { Value } from "$lib/domain/model";
  let {
    label = "",
    value,
    onchange,
  }: {
    label?: string;
    value: Value;
    onchange: (value: Value) => void;
  } = $props();
  let draft = $state(scalar(value));
  let focused = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    const next = scalar(value);
    if (!focused && next !== draft) draft = next;
  });

  function scalar(value: Value) {
    if ("text" in value) return value.text;
    if ("integer" in value) return String(value.integer);
    if ("float" in value) return String(value.float);
    return "";
  }

  function commit() {
    if ("integer" in value) {
      const next = Number.parseInt(draft, 10);
      onchange({ integer: Number.isNaN(next) ? 0 : next });
    } else if ("float" in value) {
      const next = Number.parseFloat(draft);
      onchange({ float: Number.isNaN(next) ? 0 : next });
    } else {
      onchange({ text: draft });
    }
  }

  function input(raw: string) {
    draft = raw;
    if (timer) clearTimeout(timer);
    timer = setTimeout(commit, 350);
  }

  function blur() {
    focused = false;
    if (timer) clearTimeout(timer);
    commit();
  }
</script>

<div class="field">
  {#if label}<span class="field-label">{label}</span>{/if}
  <input
    class="field-box"
    type={"integer" in value || "float" in value ? "number" : "text"}
    step={"float" in value ? "any" : undefined}
    value={draft}
    placeholder="—"
    onfocus={() => (focused = true)}
    onblur={blur}
    onkeydown={(event) =>
      event.key === "Enter" && (event.currentTarget as HTMLInputElement).blur()}
    oninput={(event) => input(event.currentTarget.value)}
  />
</div>
