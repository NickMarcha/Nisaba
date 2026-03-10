<script lang="ts">
  let {
    error = null,
    sidebar,
    main,
  } = $props<{
    error?: string | null;
    sidebar: import('svelte').Snippet;
    main: import('svelte').Snippet;
  }>();
</script>

<div class="app">
  {@render sidebar()}
  <main class="main">
    {#if error}
      <div class="error">{error}</div>
    {/if}
    {@render main()}
  </main>
</div>

<style>
  .app {
    display: flex;
    height: 100vh;
    font-family: system-ui, sans-serif;
    font-size: 14px;
  }

  .main {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: #0f0f0f;
    overflow: hidden;
  }

  .error {
    padding: 12px 20px;
    background: #2a1515;
    color: #f87171;
    font-size: 13px;
  }
</style>
