<script lang="ts">
  import { X } from "lucide-svelte";

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

  let dialog: HTMLDialogElement;

  $effect(() => {
    if (open) {
      dialog.showModal();
    } else {
      dialog.close();
    }
  });

  function confirm() {
    onconfirm();
    open = false;
  }
</script>

<dialog
  bind:this={dialog}
  class="dialog"
  onclose={() => (open = false)}
  onclick={(e) => e.target === dialog && (open = false)}
>
  <div class="header">
    <h2>{title}</h2>
    <button
      type="button"
      class="close"
      aria-label="Close"
      onclick={() => (open = false)}
    >
      <X size={16} />
    </button>
  </div>

  <p class="message">{message}</p>

  <div class="actions">
    <button
      type="button"
      class="btn-brutalist confirm"
      class:danger
      onclick={confirm}
    >
      {confirmLabel}
    </button>
  </div>
</dialog>

<style>
  .dialog {
    width: min(320px, 100%);
    padding: 20px;
    border: 2px solid var(--accent);
    border-radius: var(--radius);
    background: var(--surface-raised);
    color: var(--text);
    box-shadow: 6px 6px 0 var(--accent-shadow);
  }

  .dialog::backdrop {
    background: rgba(0, 0, 0, 0.5);
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 20px;
  }

  .header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 800;
  }

  .close {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    padding: 0;
    border: 0;
    border-radius: 4px;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
  }

  .close:hover {
    background: rgba(0, 0, 0, 0.05);
    color: var(--text);
  }

  .message {
    margin: 0 0 20px;
    font-size: 13px;
    color: var(--text-muted);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 6px;
  }

  .confirm {
    min-width: 80px;
    min-height: 32px;
    padding: 0 12px;
    font-size: 13px;
  }

  .confirm.danger {
    background: var(--danger);
    border-color: var(--danger);
  }
</style>
