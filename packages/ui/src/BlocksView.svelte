<script lang="ts">
  import type { IndexedBlock } from './types';

  let {
    blocks = [],
    blockTypeFilter = '',
    loading = false,
    onBlockClick,
    onBlockTypeFilterChange,
    fileName = (path: string) => path.replace(/\\/g, '/').split('/').pop() ?? path,
  } = $props<{
    blocks?: IndexedBlock[];
    blockTypeFilter?: string;
    loading?: boolean;
    onBlockClick?: (block: IndexedBlock) => void;
    onBlockTypeFilterChange?: () => void;
    fileName?: (path: string) => string;
  }>();
</script>

<div class="blocks-view">
  <div class="blocks-toolbar">
    <select
      class="type-filter"
      bind:value={blockTypeFilter}
      onchange={onBlockTypeFilterChange}
    >
      <option value="">All types</option>
      <option value="task">task</option>
      <option value="note">note</option>
    </select>
  </div>
  <div class="blocks-table-wrap">
    <table class="blocks-table">
      <thead>
        <tr>
          <th>Type</th>
          <th>Content</th>
          <th>File</th>
        </tr>
      </thead>
      <tbody>
        {#each blocks as block}
          <tr class="block-row" onclick={() => onBlockClick?.(block)}>
            <td>{block.block_type ?? '—'}</td>
            <td>{block.content}</td>
            <td>{fileName(block.file_path)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
  {#if blocks.length === 0 && !loading}
    <p class="empty">No blocks. Add frontmatter with <code>type:</code> and <code>block_ids:</code> to your notes.</p>
  {/if}
</div>

<style>
  .blocks-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .blocks-toolbar {
    padding: 12px 20px;
    border-bottom: 1px solid #2a2a2a;
  }

  .type-filter {
    padding: 6px 12px;
    font-size: 13px;
    background: #1a1a1a;
    color: #e2e8f0;
    border: 1px solid #3a3a3a;
    border-radius: 6px;
  }

  .blocks-table-wrap {
    flex: 1;
    overflow: auto;
    padding: 16px;
  }

  .blocks-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  .blocks-table th {
    text-align: left;
    padding: 8px 12px;
    color: #71717a;
    border-bottom: 1px solid #2a2a2a;
  }

  .blocks-table td {
    padding: 8px 12px;
    border-bottom: 1px solid #1a1a1a;
    color: #e2e8f0;
  }

  .block-row {
    cursor: pointer;
  }

  .block-row:hover {
    background: #1a1a1a;
  }

  .blocks-view .empty {
    padding: 24px;
    color: #52525b;
  }

  .blocks-view code {
    background: #252525;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 12px;
  }
</style>
