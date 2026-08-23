<script lang="ts">
  import { FileText, Trash2, Plus } from "lucide-svelte";
  import PageTypeIcon from "./PageTypeIcon.svelte";
  import { workspace } from "../state/workspace.svelte";
</script>

<aside>
  <div class="head">Inspector</div>
  <div class="content">
    {#if workspace.selection?.type === "page" && workspace.selectedPage}{@const p =
        workspace.selectedPage}
      <section>
        <div class="title"><PageTypeIcon type={p.pageType} />Page</div>
        <label
          ><span>Name</span><input
            value={p.name}
            oninput={(e) =>
              workspace.updatePage(p.id, { name: e.currentTarget.value })}
          /></label
        ><label
          ><span>Priority</span><input
            type="number"
            value={p.priority}
            oninput={(e) =>
              workspace.updatePage(p.id, {
                priority: Number(e.currentTarget.value),
              })}
          /></label
        >
        <div class="readonly">
          <span>Type</span><strong>{p.pageType}</strong>
        </div>
      </section>
      <button class="danger" onclick={() => workspace.deleteSelected()}
        ><Trash2 size={15} />Delete page</button
      >{:else if workspace.selection?.type === "entry" && workspace.selectedEntry}{@const e =
        workspace.selectedEntry}{@const p = workspace.selectedPage!}
      <section>
        <div class="title"><FileText size={16} />Entry</div>
        <label><span>ID</span><input value={e.id} readonly /></label><label
          ><span>Type</span><input
            value={e.data.entryType}
            oninput={(x) =>
              workspace.updateEntryType(p.id, e.id, x.currentTarget.value)}
          /></label
        >
      </section>
      <section>
        <div class="title">Fields</div>
        {#each Object.entries(e.data.fields) as [k, v] (k)}<label
            ><span>{k}</span><input
              value={String(v ?? "")}
              oninput={(x) =>
                workspace.updateEntryField(
                  p.id,
                  e.id,
                  k,
                  x.currentTarget.value,
                )}
            /></label
          >{/each}<button
          class="addfield"
          onclick={() => workspace.addEntryField(p.id, e.id)}
          ><Plus size={14} />Add field</button
        >
      </section>
      <button class="danger" onclick={() => workspace.deleteSelected()}
        ><Trash2 size={15} />Delete entry</button
      >{:else}<div class="empty">
        <strong>Nothing selected</strong>
        <p>Select a page or an entry to inspect it.</p>
      </div>{/if}
  </div>
</aside>

<style>
  aside {
    width: 290px;
    min-width: 260px;
    display: flex;
    flex-direction: column;
    border-left: 2px solid var(--line);
    background: var(--surface);
  }
  .head {
    height: 54px;
    display: flex;
    align-items: center;
    padding: 0 18px;
    border-bottom: 2px solid var(--line);
    font-size: 12px;
    font-weight: 900;
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }
  .content {
    flex: 1;
    overflow: auto;
    padding: 14px;
  }
  section {
    margin-bottom: 18px;
    padding-bottom: 18px;
    border-bottom: 2px solid var(--line);
  }
  .title {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 14px;
    color: var(--accent);
    font-size: 11px;
    font-weight: 900;
    text-transform: uppercase;
    letter-spacing: 0.09em;
  }
  label,
  .readonly {
    display: grid;
    gap: 6px;
    margin-bottom: 12px;
  }
  label span,
  .readonly span {
    color: var(--text-muted);
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
  }
  input {
    width: 100%;
    height: 38px;
    padding: 0 9px;
    border: 2px solid var(--border);
    background: var(--canvas);
    color: var(--text);
    outline: none;
    box-shadow: 3px 3px 0 #000;
  }
  input:focus {
    border-color: var(--accent);
  }
  input[readonly] {
    color: var(--text-muted);
  }
  .readonly strong {
    padding: 8px 10px;
    border-left: 3px solid var(--accent);
    background: var(--surface-raised);
    font-family: var(--mono);
    font-size: 12px;
    text-transform: uppercase;
  }
  .addfield,
  .danger {
    width: 100%;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    background: transparent;
    font-size: 10px;
    font-weight: 900;
    text-transform: uppercase;
    cursor: pointer;
  }
  .addfield {
    border: 2px dashed var(--border);
    color: var(--text);
  }
  .addfield:hover {
    border-color: var(--accent);
    color: var(--accent);
  }
  .danger {
    height: 38px;
    border: 2px solid var(--danger);
    color: var(--danger);
  }
  .danger:hover {
    background: var(--danger);
    color: #111;
  }
  .empty {
    display: grid;
    min-height: 180px;
    place-content: center;
    text-align: center;
  }
  .empty p {
    color: var(--text-muted);
    font-size: 12px;
  }
</style>
