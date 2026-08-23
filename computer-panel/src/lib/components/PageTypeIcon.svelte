<script lang="ts">
  import { GitBranch, Boxes, Network } from "lucide-svelte";
  import type { PageType } from "../types";
  let { type }: { type: PageType } = $props();
</script>

{#if type === "sequence"}<GitBranch
    size={16}
    strokeWidth={2.5}
  />{:else if type === "static"}<Boxes
    size={16}
    strokeWidth={2.5}
  />{:else}<Network size={16} strokeWidth={2.5} />{/if}
