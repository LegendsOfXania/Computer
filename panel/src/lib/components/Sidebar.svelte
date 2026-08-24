<script lang="ts">
  export let hover = false;
</script>

<aside class="sidebar" class:hover={hover} onmouseenter={() => hover = true} onmouseleave={() => hover = false}>
  <div class="sidebar-content">
    <slot />
  </div>
</aside>

<style>
  .sidebar {
    position: fixed;
    left: 0;
    top: 0;
    bottom: 0;
    width: 50px;
    background: var(--surface);
    border-right: 2px solid #f97316;
    cursor: pointer;

    transition: width 0.3s ease;
    overflow: hidden;
    z-index: 100;
  }

  .sidebar:hover {
    width: 250px;
  }

  .sidebar-content {
    padding: 20px;
    opacity: 0;
    transition: opacity 0.2s ease 0.1s;
  }

  .sidebar:hover .sidebar-content {
    opacity: 1;
  }
</style>
