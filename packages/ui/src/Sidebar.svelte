<script lang="ts">
  import type { IndexedBlock, ViewMode } from './types';
  import logoSvg from './lib/nisaba-logo.svg?url';

  type TreeNode = { name: string; path?: string; children?: TreeNode[] };

  function buildTree(paths: string[], pathToRelative: (p: string) => string): TreeNode[] {
    const root: Record<string, TreeNode> = {};
    for (const fullPath of paths) {
      const rel = pathToRelative(fullPath).replace(/\\/g, '/');
      const parts = rel.split('/').filter(Boolean);
      let current = root;
      for (let i = 0; i < parts.length; i++) {
        const part = parts[i]!;
        const isFile = i === parts.length - 1 && part.includes('.');
        if (!current[part]) {
          current[part] = isFile
            ? { name: part, path: fullPath }
            : { name: part, children: {} as Record<string, TreeNode> };
        }
        if (isFile) {
          (current[part] as TreeNode).path = fullPath;
        } else if (i < parts.length - 1) {
          const node = current[part] as TreeNode;
          if (!node.children) node.children = {};
          current = node.children as Record<string, TreeNode>;
        }
      }
    }
    function toArray(obj: Record<string, TreeNode>): TreeNode[] {
      return Object.entries(obj)
        .map(([k, v]) => ({ ...v, name: v.name || k }))
        .sort((a, b) => {
          const aIsFile = !!a.path;
          const bIsFile = !!b.path;
          if (aIsFile !== bIsFile) return aIsFile ? 1 : -1;
          return a.name.localeCompare(b.name);
        });
    }
    function recurse(n: TreeNode): TreeNode {
      if (n.children && typeof n.children === 'object' && !Array.isArray(n.children)) {
        n.children = toArray(n.children as Record<string, TreeNode>).map(recurse);
      }
      return n;
    }
    return toArray(root).map(recurse);
  }

  let {
    header,
    syncRow,
    files = [],
    selectedPath = null,
    viewMode = 'notes',
    loading = false,
    blockTypeFilter = '',
    blocks = [],
    renamingPath = null,
    renameValue = $bindable(''),
    contextMenu = null,
    onSelectFile,
    onSelectBlock,
    onRename,
    onRenameClick,
    onCreateNote,
    onContextMenu,
    onViewModeChange,
    onBlockTypeFilterChange,
    onRenameBlur,
    onContextMenuClose,
    fileName = (path: string) => path.replace(/\\/g, '/').split('/').pop() ?? path,
    pathToRelative = (path: string) => path.replace(/\\/g, '/'),
    treeView = false,
    isFileSynced,
    syncIndicatorEnabled = false,
  } = $props<{
    header?: import('svelte').Snippet;
    syncRow?: import('svelte').Snippet;
    files?: string[];
    selectedPath?: string | null;
    viewMode?: ViewMode;
    loading?: boolean;
    blockTypeFilter?: string;
    blocks?: IndexedBlock[];
    renamingPath?: string | null;
    renameValue?: string;
    contextMenu?: { x: number; y: number; path: string | null } | null;
    onSelectFile?: (path: string) => void;
    onSelectBlock?: (block: IndexedBlock) => void;
    onRename?: (path: string, newName: string) => void;
    onRenameClick?: (path: string) => void;
    onCreateNote?: () => void;
    onContextMenu?: (e: MouseEvent) => void;
    onViewModeChange?: (mode: ViewMode) => void;
    onBlockTypeFilterChange?: () => void;
    onRenameBlur?: () => void;
    onContextMenuClose?: () => void;
    fileName?: (path: string) => string;
    pathToRelative?: (path: string) => string;
    treeView?: boolean;
    isFileSynced?: (path: string) => boolean;
    syncIndicatorEnabled?: boolean;
  }>();

  let expandedFolders = $state<Set<string>>(new Set());

  const fileTree = $derived(treeView ? buildTree(files, pathToRelative) : []);

  $effect(() => {
    if (!treeView || !selectedPath) return;
    const rel = pathToRelative(selectedPath).replace(/\\/g, '/');
    const parts = rel.split('/').filter(Boolean);
    if (parts.length <= 1) return;
    const toAdd = parts.slice(0, -1).map((_, i) => parts.slice(0, i + 1).join('/'));
    const hasNew = toAdd.some((p) => !expandedFolders.has(p));
    if (hasNew) {
      expandedFolders = new Set([...expandedFolders, ...toAdd]);
    }
  });

  function toggleFolder(relPath: string) {
    expandedFolders = new Set(expandedFolders);
    if (expandedFolders.has(relPath)) {
      expandedFolders.delete(relPath);
    } else {
      expandedFolders.add(relPath);
    }
  }

  function isUnsynced(path: string): boolean {
    return syncIndicatorEnabled && isFileSynced !== undefined && !isFileSynced(path);
  }
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <span class="brand">
      <img src={logoSvg} alt="" class="brand-logo" />
      Nisaba
    </span>
    {#if header}
      {@render header()}
    {/if}
  </div>
  {#if syncRow}
    <div class="sync-row">
      {@render syncRow()}
    </div>
  {/if}
  {#if files.length > 0 || loading}
    <div class="view-tabs">
      <button
        class="view-tab"
        class:active={viewMode === 'notes'}
        onclick={() => onViewModeChange?.('notes')}
      >
        Notes
      </button>
      <button
        class="view-tab"
        class:active={viewMode === 'blocks'}
        onclick={() => onViewModeChange?.('blocks')}
      >
        Blocks
      </button>
      <button
        class="view-tab"
        class:active={viewMode === 'links'}
        onclick={() => onViewModeChange?.('links')}
      >
        Links
      </button>
    </div>
    <div
      class="file-list"
      class:hidden={viewMode !== 'notes' && viewMode !== 'links'}
      oncontextmenu={onContextMenu}
    >
      {#if treeView && fileTree.length > 0}
        {#each fileTree as node}
          {@render treeNode(node, '')}
        {/each}
      {:else}
        {#each files as path}
          {@render fileItem(path)}
        {/each}
      {/if}
      {#if files.length === 0 && !loading}
        <p class="empty">Right-click to create a note</p>
      {/if}
    </div>

    {#snippet treeNode(n: TreeNode, relPath: string)}
      {#if n.path}
        <div class="tree-file-wrapper" data-path={n.path}>
          {#if renamingPath === n.path}
            <form
              class="rename-form tree-rename"
              data-path={n.path}
              onsubmit={(e) => {
                e.preventDefault();
                onRename?.(n.path!, renameValue);
              }}
            >
              <input
                type="text"
                bind:value={renameValue}
                autofocus
                onblur={() => onRenameBlur?.(n.path!)}
                onkeydown={(e) => e.key === 'Escape' && onRenameBlur?.()}
              />
            </form>
          {:else}
            <button
              class="file-item tree-file"
              class:selected={n.path === selectedPath}
              class:unsynced={isUnsynced(n.path)}
              data-path={n.path}
              title={isUnsynced(n.path) ? 'File will be synced on first open' : undefined}
              onclick={() => onSelectFile?.(n.path!)}
            >
              {#if isUnsynced(n.path)}
                <span class="unsynced-icon" aria-hidden="true">
                  <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="#f1e60e" width="14" height="14"><path d="M15.9375 6.11972C17.7862 7.39969 19 9.55585 19 12C19 15.9274 15.866 19.1111 12 19.1111C11.6411 19.1111 11.2885 19.0837 10.9441 19.0307M13.0149 4.96309C12.6836 4.9142 12.3447 4.88889 12 4.88889C8.13401 4.88889 5 8.07264 5 12C5 14.4071 6.17734 16.5349 7.97895 17.8215M13.0149 4.96309L12.4375 4M13.0149 4.96309L12.4375 5.77778M10.9441 19.0307L11.7866 18.2222M10.9441 19.0307L11.5625 20M12 9V12.5M12 14.5V15" stroke="#f1e60e" stroke-linecap="round" stroke-linejoin="round"/></svg>
                </span>
              {/if}
              {fileName(n.path)}
            </button>
          {/if}
        </div>
      {:else}
        {@const folderPath = relPath ? `${relPath}/${n.name}` : n.name}
        {@const isExpanded = expandedFolders.has(folderPath)}
        <div class="tree-folder">
          <button
            class="tree-folder-toggle"
            onclick={() => toggleFolder(folderPath)}
            aria-expanded={isExpanded}
          >
            <span class="tree-chevron" class:expanded={isExpanded}>▾</span>
            {n.name}
          </button>
          {#if isExpanded && n.children?.length}
            <div class="tree-children">
              {#each n.children as child}
                {@render treeNode(child, folderPath)}
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    {/snippet}

    {#snippet fileItem(path: string)}
      {#if renamingPath === path}
        <form
          class="rename-form"
          data-path={path}
          onsubmit={(e) => {
            e.preventDefault();
            onRename?.(path, renameValue);
          }}
        >
          <input
            type="text"
            bind:value={renameValue}
            autofocus
            onblur={() => onRenameBlur?.(path)}
            onkeydown={(e) => e.key === 'Escape' && onRenameBlur?.()}
          />
        </form>
      {:else}
        <button
          class="file-item"
          class:selected={path === selectedPath}
          class:unsynced={isUnsynced(path)}
          data-path={path}
          title={isUnsynced(path) ? 'File will be synced on first open' : undefined}
          onclick={() => onSelectFile?.(path)}
        >
          {#if isUnsynced(path)}
            <span class="unsynced-icon" aria-hidden="true">
              <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="#f1e60e" width="14" height="14"><path d="M15.9375 6.11972C17.7862 7.39969 19 9.55585 19 12C19 15.9274 15.866 19.1111 12 19.1111C11.6411 19.1111 11.2885 19.0837 10.9441 19.0307M13.0149 4.96309C12.6836 4.9142 12.3447 4.88889 12 4.88889C8.13401 4.88889 5 8.07264 5 12C5 14.4071 6.17734 16.5349 7.97895 17.8215M13.0149 4.96309L12.4375 4M13.0149 4.96309L12.4375 5.77778M10.9441 19.0307L11.7866 18.2222M10.9441 19.0307L11.5625 20M12 9V12.5M12 14.5V15" stroke="#f1e60e" stroke-linecap="round" stroke-linejoin="round"/></svg>
            </span>
          {/if}
          {fileName(path)}
        </button>
      {/if}
    {/snippet}
    {#if contextMenu}
      <div class="context-menu-backdrop" onclick={onContextMenuClose} role="button" tabindex="-1"></div>
      <div
        class="context-menu"
        style="left: {contextMenu.x}px; top: {contextMenu.y}px"
      >
        <button class="context-item" onclick={onCreateNote}>
          New note
        </button>
        {#if contextMenu.path}
          <button
            class="context-item"
            onclick={() => {
              onRenameClick?.(contextMenu!.path!);
              onContextMenuClose?.();
            }}
          >
            Rename
          </button>
        {/if}
      </div>
    {/if}
  {:else}
    <p class="empty">Open a folder to browse notes</p>
  {/if}
</aside>

<style>
  .sidebar {
    width: 248px;
    flex-shrink: 0;
    background: #1a1a1a;
    border-right: 1px solid #2a2a2a;
    display: flex;
    flex-direction: column;
  }

  .sidebar-header {
    padding: 16px;
    border-bottom: 1px solid #2a2a2a;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    min-width: 0;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    color: #f5f5f5;
  }

  .brand-logo {
    width: 24px;
    height: 24px;
    flex-shrink: 0;
    object-fit: contain;
  }

  .sync-row {
    padding: 8px 16px;
    border-bottom: 1px solid #2a2a2a;
    min-width: 0;
    overflow: hidden;
  }

  .file-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
  }

  .file-item {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 8px 16px;
    text-align: left;
    font-size: 13px;
    color: #a1a1aa;
    background: none;
    border: none;
    cursor: pointer;
  }

  .file-item:hover {
    background: #252525;
    color: #f5f5f5;
  }

  .file-item.selected {
    background: #2a2a2a;
    color: #f5f5f5;
  }

  .file-item.unsynced {
    background: rgba(253, 244, 140, 0.15);
  }

  .file-item.unsynced:hover,
  .file-item.unsynced.selected {
    background: rgba(253, 244, 140, 0.25);
  }

  .unsynced-icon {
    display: inline-flex;
    margin-right: 6px;
    flex-shrink: 0;
    vertical-align: middle;
  }

  .tree-folder {
    padding-left: 0;
  }

  .tree-folder-toggle {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
    padding: 6px 16px 6px 12px;
    text-align: left;
    font-size: 13px;
    color: #a1a1aa;
    background: none;
    border: none;
    cursor: pointer;
  }

  .tree-folder-toggle:hover {
    background: #252525;
    color: #f5f5f5;
  }

  .tree-chevron {
    font-size: 10px;
    opacity: 0.7;
    transition: transform 0.15s ease;
  }

  .tree-chevron.expanded {
    transform: rotate(0deg);
  }

  .tree-chevron:not(.expanded) {
    transform: rotate(-90deg);
  }

  .tree-children {
    padding-left: 12px;
    border-left: 1px solid #2a2a2a;
    margin-left: 12px;
  }

  .tree-file-wrapper {
    padding-left: 12px;
  }

  .tree-file {
    padding-left: 12px;
  }

  .tree-rename {
    padding-left: 12px;
  }

  .empty {
    padding: 16px;
    color: #71717a;
    font-size: 13px;
  }

  .rename-form {
    padding: 4px 16px;
  }

  .rename-form input {
    width: 100%;
    padding: 6px 8px;
    font-size: 13px;
    background: #252525;
    color: #f5f5f5;
    border: 1px solid #3a3a3a;
    border-radius: 4px;
  }

  .context-menu-backdrop {
    position: fixed;
    inset: 0;
    z-index: 100;
  }

  .context-menu {
    position: fixed;
    z-index: 101;
    min-width: 140px;
    padding: 4px 0;
    background: #1a1a1a;
    border: 1px solid #2a2a2a;
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  }

  .context-item {
    display: block;
    width: 100%;
    padding: 8px 16px;
    text-align: left;
    font-size: 13px;
    color: #e2e8f0;
    background: none;
    border: none;
    cursor: pointer;
  }

  .context-item:hover {
    background: #252525;
  }

  .view-tabs {
    display: flex;
    padding: 8px;
    gap: 4px;
    border-bottom: 1px solid #2a2a2a;
  }

  .view-tab {
    flex: 1;
    padding: 6px 12px;
    font-size: 13px;
    background: #252525;
    color: #a1a1aa;
    border: 1px solid #2a2a2a;
    border-radius: 6px;
    cursor: pointer;
  }

  .view-tab:hover {
    background: #2a2a2a;
    color: #f5f5f5;
  }

  .view-tab.active {
    background: #333;
    color: #f5f5f5;
  }

  .file-list.hidden {
    display: none;
  }
</style>
