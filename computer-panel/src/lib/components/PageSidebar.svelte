<script lang="ts">
  import { Plus } from "lucide-svelte";
  import PageTypeIcon from "./PageTypeIcon.svelte";
  import { workspace } from "../state/workspace.svelte";
  let creating = $state(false);
  let name = $state("");
  function submit() {
    if (!name.trim()) return;
    workspace.createPage(name.trim());
    name = "";
    creating = false;
  }
</script>

<aside>
  <div class="head">
    Pages<button onclick={() => (creating = true)}><Plus size={16} /></button>
  </div>
  <div class="pages">
    {#each workspace.pages as page (page.id)}<button
        class:selected={workspace.selection?.pageId === page.id}
        class="page"
        onclick={() => workspace.selectPage(page.id)}
        ><PageTypeIcon type={page.pageType} /><span>{page.name}</span><small
          >{page.priority}</small
        ></button
      >{:else}<p>No pages</p>{/each}
  </div>
  <div class="foot">
    {#if creating}<form
        onsubmit={(e) => {
          e.preventDefault();
          submit();
        }}
      >
        <input bind:value={name} placeholder="Page name" autofocus /><button
          type="submit">Create</button
        >
      </form>{:else}<button class="create" onclick={() => (creating = true)}
        ><Plus size={16} />Create page</button
      >{/if}
  </div>
</aside>

<style>
  aside {
    width: 260px;
    min-width: 230px;
    display: flex;
    flex-direction: column;
    border-right: 2px solid var(--line);
    background: var(--surface);
  }
  .head {
    height: 54px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 14px 0 18px;
    border-bottom: 2px solid var(--line);
    font-size: 12px;
    font-weight: 900;
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }
  .head button {
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    border: 2px solid var(--border);
    background: transparent;
    color: var(--text);
    cursor: pointer;
  }
  .head button:hover {
    background: var(--accent);
    color: #111;
  }
  .pages {
    flex: 1;
    overflow: auto;
    padding: 8px;
  }
  .page {
    width: 100%;
    height: 42px;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 10px;
    border: 2px solid transparent;
    background: transparent;
    color: var(--text);
    text-align: left;
    cursor: pointer;
  }
  .page:hover {
    background: var(--surface-raised);
    border-color: var(--line);
  }
  .page.selected {
    background: var(--accent);
    border-color: var(--border);
    color: #111;
    box-shadow: 3px 3px 0 #000;
  }
  .page span {
    flex: 1;
    font-size: 14px;
    font-weight: 700;
  }
  .page small {
    font-family: var(--mono);
    opacity: 0.65;
  }
  .foot {
    padding: 10px;
    border-top: 2px solid var(--line);
  }
  .create {
    width: 100%;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border: 2px solid var(--border);
    background: var(--accent);
    color: #111;
    font-size: 11px;
    font-weight: 900;
    text-transform: uppercase;
    box-shadow: 4px 4px 0 #000;
    cursor: pointer;
  }
  .create:active {
    transform: translate(4px, 4px);
    box-shadow: none;
  }
  form {
    display: flex;
    gap: 6px;
  }
  input {
    min-width: 0;
    flex: 1;
    height: 38px;
    padding: 0 8px;
    border: 2px solid var(--border);
    background: var(--canvas);
    color: var(--text);
    outline: none;
  }
  form button {
    border: 2px solid var(--border);
    background: var(--accent);
    font-weight: 800;
    cursor: pointer;
  }
  p {
    color: var(--text-muted);
    text-align: center;
    font-size: 13px;
  }
</style>
