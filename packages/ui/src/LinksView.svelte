<script lang="ts">
  import type { IndexedLink } from './types';

  let {
    links = [],
    focusPath = null,
    loading = false,
    onLinkClick,
    fileName = (path: string) => path.replace(/\\/g, '/').split('/').pop() ?? path,
  } = $props<{
    links?: IndexedLink[];
    focusPath?: string | null;
    loading?: boolean;
    onLinkClick?: (path: string) => void;
    fileName?: (path: string) => string;
  }>();

  const outbound = $derived(
    focusPath ? links.filter((l) => l.source_file === focusPath) : links
  );
  const inbound = $derived(
    focusPath ? links.filter((l) => l.target === focusPath) : []
  );
</script>

<div class="links-view">
  <div class="links-toolbar">
    {#if focusPath}
      <span class="focus-label">Links for: {fileName(focusPath)}</span>
    {:else}
      <span class="focus-label">All links</span>
    {/if}
  </div>
  <div class="links-table-wrap">
    <table class="links-table">
      <thead>
        <tr>
          <th>Relation</th>
          <th>Source</th>
          <th>Target</th>
        </tr>
      </thead>
      <tbody>
        {#each outbound as link}
          <tr
            class="link-row"
            onclick={() => onLinkClick?.(link.target)}
            role="button"
          >
            <td>{link.relation_key ?? 'link'}</td>
            <td>{fileName(link.source_file)}</td>
            <td>{link.target}</td>
          </tr>
        {/each}
        {#if focusPath && inbound.length > 0}
          <tr class="section-row">
            <td colspan="3">Backlinks</td>
          </tr>
          {#each inbound as link}
            <tr
              class="link-row"
              onclick={() => onLinkClick?.(link.source_file)}
              role="button"
            >
              <td>{link.relation_key ?? 'link'}</td>
              <td>{fileName(link.source_file)}</td>
              <td>{link.target}</td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>
  {#if links.length === 0 && !loading}
    <p class="empty">
      No links. Add <code>[[wikilinks]]</code> in body or
      <code>assignee: [[Alice]]</code> in frontmatter.
    </p>
  {/if}
</div>

<style>
  .links-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .links-toolbar {
    padding: 12px 20px;
    border-bottom: 1px solid #2a2a2a;
  }

  .focus-label {
    font-size: 13px;
    color: #a1a1aa;
  }

  .links-table-wrap {
    flex: 1;
    overflow: auto;
    padding: 16px;
  }

  .links-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  .links-table th {
    text-align: left;
    padding: 8px 12px;
    color: #71717a;
    border-bottom: 1px solid #2a2a2a;
  }

  .links-table td {
    padding: 8px 12px;
    border-bottom: 1px solid #1a1a1a;
    color: #e2e8f0;
  }

  .link-row {
    cursor: pointer;
  }

  .link-row:hover {
    background: #1a1a1a;
  }

  .section-row {
    color: #71717a;
    font-weight: 500;
  }

  .section-row td {
    padding-top: 12px;
  }

  .links-view .empty {
    padding: 24px;
    color: #52525b;
  }

  .links-view code {
    background: #252525;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 12px;
  }
</style>
