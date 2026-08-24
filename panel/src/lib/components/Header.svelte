<script>
  import { Copy } from "lucide-svelte";

  let isPublished = $state(false);

  function togglePublish() {
    isPublished = !isPublished;
  }

  async function copyPageId() {
    await navigator.clipboard.writeText("page-id");
  }
</script>

<header class="header">
  <div class="informations">
    <span class="page-icon">PageIcon</span>
    <span class="page-name">PageName</span>
    <button type="button" class="page-id" onclick={copyPageId}>
      <span>page-id</span>
      <Copy size={12} />
    </button>
  </div>

  <div class="actions">
    <div class="connection">
      <span class="status connected"></span>
      <span>Online</span>
    </div>

    <button class="publish" class:active={isPublished} onclick={togglePublish}>
      <span>{isPublished ? "Published" : "Staging"}</span>
    </button>
  </div>
</header>

<style>
  .header {
    height: 50px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 50px;
    border-bottom: 2px solid #f97316;
    background: var(--surface);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 20px;
  }

  .connection {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-muted);
    font-size: 13px;
    font-weight: 600;
  }

  .status {
    width: 10px;
    height: 10px;
    border: 1px solid var(--border);
    background: var(--danger);
  }

  .status.connected {
    background: #059669;
  }

  .publish {
    display: flex;
    align-items: center;
    justify-content: center;

    min-height: 34px;
    min-width: 95px;
    padding: 0 14px;

    border: none;
    border-radius: var(--radius);

    font-size: 12px;
    font-weight: 800;
    cursor: pointer;

    color: #1c1917;
    background: #f97316;

    transform: translate(-2px, -2px);
    box-shadow: 3px 3px 0px #c2410c;

    transition:
      transform 0.15s ease,
      box-shadow 0.15s ease,
      background-color 0.15s ease;
  }

  .publish:hover {
    background: #fb923c;
    transform: translate(-3px, -3px);
    box-shadow: 4px 4px 0px #c2410c;
  }

  .publish.active {
    background: #059669;
    transform: translate(1px, 1px);
    box-shadow: 0px 0px 0px transparent;
  }

  .publish.active:hover {
    background: #10b981;
    transform: translate(1px, 1px);
    box-shadow: 0px 0px 0px transparent;
  }

  .informations {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .page-id {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 2px 6px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.08);
    color: var(--text-muted);
    font-size: 12px;
    font-family: monospace;
    font-weight: 400;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .page-id:hover {
    background: rgba(0, 0, 0, 0.12);
  }
</style>
