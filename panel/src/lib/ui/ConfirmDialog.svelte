<script lang="ts">
  import { X } from "lucide-svelte";
  import Dialog from "./Dialog.svelte";
  let {
    open = $bindable(false),
    title = "Confirm",
    message,
    confirmLabel = "Confirm",
    danger = false,
    onconfirm,
  }: {
    open?: boolean;
    title?: string;
    message: string;
    confirmLabel?: string;
    danger?: boolean;
    onconfirm: () => void;
  } = $props();
  function confirm() {
    onconfirm();
    open = false;
  }
</script>

<Dialog bind:open>
  <div class="header">
    <h2>{title}</h2>
    <button
      type="button"
      class="close"
      aria-label="Close"
      onclick={() => (open = false)}><X size={16} /></button
    >
  </div>
  <p class="message">{message}</p>
  <div class="actions">
    <button type="button" onclick={() => (open = false)}>Cancel</button>
    <button
      type="button"
      class="btn-brutalist"
      class:danger
      onclick={confirm}
      data-autofocus>{confirmLabel}</button
    >
  </div>
</Dialog>
