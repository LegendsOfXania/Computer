<script lang="ts">
  import { Cpu, ChevronRight } from "lucide-svelte";
  import { workspace } from "../state/workspace.svelte";
</script>

<header>
  <div class="brand">
    <span class="icon"><Cpu size={20} strokeWidth={3} /></span><strong
      >Computer</strong
    >
  </div>
  <div class="right">
    <span
      class:online={workspace.connection === "connected"}
      class:broken={workspace.connection === "disconnected"}
      class="status"
      ><i></i>{workspace.connection === "connected"
        ? "Online"
        : workspace.connection === "connecting"
          ? "Connecting"
          : "Connection lost"}</span
    ><button>Staging <ChevronRight size={14} /> <b>Production</b></button>
  </div>
</header>

<style>
  header {
    height: 58px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 2px solid var(--line);
    background: var(--surface);
  }
  .brand {
    height: 100%;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 20px;
    border-right: 2px solid var(--line);
    font-size: 18px;
  }
  .icon {
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    background: var(--accent);
    color: #111;
    border: 2px solid var(--border);
    box-shadow: 3px 3px 0 #000;
  }
  .right {
    display: flex;
    align-items: center;
    gap: 18px;
    padding-right: 16px;
  }
  .status {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
    font-weight: 900;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-muted);
  }
  i {
    width: 9px;
    height: 9px;
    border: 2px solid var(--border);
    background: var(--warning);
    box-shadow: 2px 2px 0 #000;
  }
  .online i {
    background: var(--success);
  }
  .broken i {
    background: var(--danger);
  }
  button {
    height: 34px;
    display: flex;
    align-items: center;
    gap: 6px;
    border: 2px solid var(--border);
    background: var(--surface-raised);
    color: var(--text);
    padding: 0 10px;
    font-size: 10px;
    font-weight: 900;
    text-transform: uppercase;
    box-shadow: 3px 3px 0 #000;
    cursor: pointer;
  }
  button:active {
    transform: translate(3px, 3px);
    box-shadow: none;
  }
  b {
    color: var(--accent);
  }
</style>
