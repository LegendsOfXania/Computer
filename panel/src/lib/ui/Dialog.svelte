<script lang="ts">
  import { tick, type Snippet } from "svelte";

  let {
    open = $bindable(false),
    initialFocus = "[data-autofocus]",
    class: className = "",
    children,
  }: {
    open?: boolean;
    initialFocus?: string;
    class?: string;
    children: Snippet;
  } = $props();

  let dialog: HTMLDialogElement;
  let restoreFocus: HTMLElement | null = null;

  function show() {
    restoreFocus =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    if (!dialog.open) dialog.showModal();
    tick().then(() => {
      const target =
        dialog.querySelector<HTMLElement>(initialFocus) ??
        dialog.querySelector<HTMLElement>("button,input,select,textarea");
      target?.focus();
    });
  }

  function close() {
    if (dialog.open) dialog.close();
    else open = false;
  }

  $effect(() => {
    if (open) show();
    else close();
  });

  function handleClose() {
    open = false;
    restoreFocus?.focus();
    restoreFocus = null;
  }
</script>

<dialog
  bind:this={dialog}
  class={`dialog ${className}`}
  onclose={handleClose}
  onclick={(event) => event.target === dialog && dialog.close()}
>
  {@render children()}
</dialog>
