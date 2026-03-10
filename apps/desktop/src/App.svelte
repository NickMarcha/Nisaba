<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import * as Y from 'yjs';
  import { WebsocketProvider } from 'y-websocket';
  import {
    AppLayout,
    Sidebar,
    EditorPane,
    Editor,
    EditorSync,
    BlocksView,
    LinksView,
    Placeholder,
    type IndexedBlock,
    type IndexedLink,
  } from '@nisaba/ui';
  import { getSyncUrl, setSyncUrl, connectSync, whenSynced, listServerFiles, readFileFromServer } from './sync';
  import {
    getLastVault,
    getRecentVaults,
    setLastVault,
    removeRecentVault,
  } from './vaultStorage';
  import { onMount, onDestroy } from 'svelte';

  let vaultPath = $state<string | null>(null);
  let files = $state<string[]>([]);
  let serverDocIds = $state<Set<string>>(new Set());
  let selectedPath = $state<string | null>(null);
  let content = $state('');
  let editorContent = $state('');
  let loading = $state(false);
  let error = $state<string | null>(null);
  let viewMode = $state<'notes' | 'blocks' | 'links'>('notes');
  let blocks = $state<IndexedBlock[]>([]);
  let links = $state<IndexedLink[]>([]);
  let blockTypeFilter = $state('');
  let editorMode = $state<'raw' | 'edit' | 'split' | 'read'>('edit');
  let editorRef = $state<Editor | InstanceType<typeof EditorSync> | null>(null);
  let contextMenu = $state<{ x: number; y: number; path: string | null } | null>(null);
  let syncUrl = $state(getSyncUrl());
  let syncDoc = $state<Y.Doc | null>(null);
  let syncProvider = $state<WebsocketProvider | null>(null);

  $effect(() => {
    if (!syncUrl.trim() && (syncProvider || syncDoc)) {
      syncProvider?.destroy();
      syncProvider = null;
      syncDoc?.destroy();
      syncDoc = null;
    }
  });

  function docPathFromVault(fullPath: string): string {
    if (!vaultPath) return fullPath.replace(/\\/g, '/');
    const base = vaultPath.replace(/\\/g, '/').replace(/\/$/, '');
    const full = fullPath.replace(/\\/g, '/');
    return full.startsWith(base) ? full.slice(base.length).replace(/^\//, '') : full;
  }
  let renamingPath = $state<string | null>(null);
  let renameValue = $state('');
  let vaultMenuOpen = $state(false);
  let recentVaults = $state<string[]>([]);

  async function openVault() {
    error = null;
    vaultMenuOpen = false;
    const path = await invoke<string | null>('open_vault');
    if (path) {
      await openVaultByPath(path);
    }
  }

  async function openVaultByPath(path: string) {
    if (!path?.trim()) return;
    error = null;
    vaultMenuOpen = false;
    loading = true;
    try {
      files = await invoke<string[]>('list_vault_files', { vaultPath: path });
      vaultPath = path;
      setLastVault(path);
      recentVaults = getRecentVaults();
      await invoke('index_vault', { vaultPath: path });
      await invoke('watch_vault', { vaultPath: path });
      selectedPath = null;
      content = '';
      if (viewMode === 'blocks') await loadBlocks();
      if (viewMode === 'links') await loadLinks();
    } catch (e) {
      error = String(e);
      removeRecentVault(path);
      recentVaults = getRecentVaults();
    } finally {
      loading = false;
    }
  }

  let unlistenVaultChanged: (() => void) | null = null;
  let unlistenCloseRequested: (() => void) | null = null;

  onMount(() => {
    recentVaults = getRecentVaults();
    const last = getLastVault();
    if (last?.trim()) {
      openVaultByPath(last);
    }
    listen('vault-files-changed', () => {
      refreshFiles();
    }).then((fn) => (unlistenVaultChanged = fn));

    getCurrentWindow().onCloseRequested(async (event) => {
      event.preventDefault();
      unlistenVaultChanged?.();
      unlistenVaultChanged = null;
      const currentContent = editorRef?.getContent?.() ?? editorContent;
      if (selectedPath && currentContent !== content) {
        try {
          await invoke('write_file', { path: selectedPath, content: currentContent });
          if (vaultPath) await invoke('index_vault', { vaultPath });
        } catch {
          // Best effort; app is closing
        }
      }
      syncProvider?.destroy();
      syncProvider = null;
      syncDoc?.destroy();
      syncDoc = null;
      getCurrentWindow().destroy();
    }).then((fn) => (unlistenCloseRequested = fn));
  });

  onDestroy(() => {
    unlistenVaultChanged?.();
    unlistenCloseRequested?.();
  });

  async function refreshFiles() {
    if (!vaultPath) return;
    try {
      const localFiles = await invoke<string[]>('list_vault_files', { vaultPath });
      let merged = [...localFiles];
      if (syncUrl.trim()) {
        try {
          const serverIds = await listServerFiles(syncUrl);
          serverDocIds = new Set(serverIds);
          const base = vaultPath.replace(/\\/g, '/').replace(/\/$/, '');
          for (const docId of serverIds) {
            const fullPath = `${base}/${docId.replace(/\\/g, '/')}`;
            if (!merged.includes(fullPath)) merged.push(fullPath);
          }
          merged.sort();
        } catch {
          serverDocIds = new Set();
        }
      } else {
        serverDocIds = new Set();
      }
      files = merged;
      await invoke('index_vault', { vaultPath });
      if (viewMode === 'blocks') await loadBlocks();
      if (viewMode === 'links') await loadLinks();
    } catch (e) {
      error = String(e);
    }
  }

  $effect(() => {
    if (!vaultPath || !syncUrl.trim()) return;
    const id = setInterval(refreshFiles, 4000);
    return () => clearInterval(id);
  });

  async function loadFiles() {
    if (!vaultPath) return;
    loading = true;
    error = null;
    try {
      await refreshFiles();
      selectedPath = null;
      content = '';
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function loadBlocks() {
    if (!vaultPath) return;
    try {
      blocks = await invoke<IndexedBlock[]>('query_blocks', {
        vaultPath,
        blockType: blockTypeFilter || null,
      });
    } catch (e) {
      blocks = [];
    }
  }

  async function selectFile(path: string) {
    if (selectedPath) {
      const currentContent = editorRef?.getContent?.() ?? editorContent;
      if (currentContent !== content) {
        await saveFile(currentContent);
      }
    }
    if (syncProvider) {
      syncProvider.destroy();
      syncProvider = null;
    }
    if (syncDoc) {
      syncDoc.destroy();
      syncDoc = null;
    }

    loading = true;
    error = null;
    try {
      let newContent: string;
      try {
        newContent = await invoke<string>('read_file', { path });
      } catch {
        // File may be server-only; fetch and create locally if sync enabled
        if (syncUrl.trim() && vaultPath) {
          const docId = docPathFromVault(path);
          newContent = await readFileFromServer(syncUrl, docId);
          await invoke('write_file', { path, content: newContent });
        } else {
          throw new Error('File not found');
        }
      }
      selectedPath = path;
      content = newContent;
      editorContent = newContent;

      if (syncUrl.trim()) {
        const ydoc = new Y.Doc();
        const ytext = ydoc.getText('content');
        const docPath = docPathFromVault(path);
        const provider = connectSync(syncUrl, docPath, ydoc);
        // Wait for sync before seeding. Do NOT set syncDoc/syncProvider yet:
        // EditorSync would mount and seed with stale local content, then server
        // state would merge in → duplicated content.
        await whenSynced(provider);
        if (ytext.length === 0 && newContent) {
          ydoc.transact(() => ytext.insert(0, newContent));
        }
        content = ytext.toString();
        editorContent = content;
        // Persist server content to disk so local stays in sync (e.g. after web edits)
        if (content !== newContent) {
          try {
            await invoke('write_file', { path, content });
            if (vaultPath) await invoke('index_vault', { vaultPath });
          } catch {
            // Best effort; user can save manually
          }
        }
        syncDoc = ydoc;
        syncProvider = provider;
      }
    } catch (e) {
      error = String(e);
      selectedPath = path;
      content = '';
      editorContent = '';
    } finally {
      loading = false;
    }
  }

  async function saveFile(value: string) {
    if (!selectedPath) return;
    content = value;
    editorContent = value;
    try {
      await invoke('write_file', { path: selectedPath, content: value });
      error = null;
      if (vaultPath) await invoke('index_vault', { vaultPath });
      if (viewMode === 'blocks') await loadBlocks();
      if (viewMode === 'links') await loadLinks();
    } catch (e) {
      error = String(e);
    }
  }

  async function loadLinks() {
    if (!vaultPath) return;
    try {
      links = await invoke<IndexedLink[]>('query_links', {
        vaultPath,
        sourceFile: null,
        target: null,
      });
    } catch (e) {
      links = [];
    }
  }

  function resolveTargetToPath(target: string): string | null {
    if (files.includes(target)) return target;
    const norm = (p: string) => p.replace(/\\/g, '/').replace(/\.md$/i, '');
    const targetNorm = target.replace(/\\/g, '/');
    for (const f of files) {
      const fn = norm(f);
      if (fn === targetNorm || fn.endsWith('/' + targetNorm)) return f;
    }
    const withExt = targetNorm + (targetNorm.endsWith('.md') ? '' : '.md');
    for (const f of files) {
      if (f.replace(/\\/g, '/').endsWith(withExt)) return f;
    }
    return null;
  }

  function fileName(path: string): string {
    const parts = path.replace(/\\/g, '/').split('/');
    return parts[parts.length - 1] ?? path;
  }

  async function doRename(oldPath: string, newName: string) {
    if (!vaultPath || !newName.trim()) return;
    const base = vaultPath.replace(/\\/g, '/').replace(/\/$/, '');
    const newPath = `${base}/${newName.trim().endsWith('.md') ? newName.trim() : newName.trim() + '.md'}`;
    if (newPath === oldPath) return;
    try {
      await invoke('rename_file', { oldPath, newPath });
      if (selectedPath === oldPath) selectedPath = newPath;
      await loadFiles();
    } catch (e) {
      error = String(e);
    }
    renamingPath = null;
  }

  async function createNote() {
    if (!vaultPath) return;
    const base = vaultPath.replace(/\\/g, '/').replace(/\/$/, '');
    const exists = (name: string) =>
      files.some((f) => f.replace(/\\/g, '/').endsWith('/' + name));
    let name = 'untitled.md';
    let n = 1;
    while (exists(name)) {
      name = `untitled-${n}.md`;
      n++;
    }
    const path = `${base}/${name}`;
    try {
      await invoke('write_file', { path, content: '# Untitled\n\n' });
      await loadFiles();
      await selectFile(path);
    } catch (e) {
      error = String(e);
    }
  }

  async function saveBeforeModeChange() {
    const c = editorRef?.getContent?.() ?? editorContent;
    if (c !== content && selectedPath) await saveFile(c);
  }
</script>

<AppLayout {error}>
  {#snippet sidebar()}
    <Sidebar
      files={files}
      selectedPath={selectedPath}
      {viewMode}
      {loading}
      {blockTypeFilter}
      {blocks}
      {renamingPath}
      bind:renameValue
      {contextMenu}
      treeView={true}
      pathToRelative={docPathFromVault}
      isFileSynced={(path) => serverDocIds.has(docPathFromVault(path))}
      syncIndicatorEnabled={!!syncUrl.trim()}
      onSelectFile={selectFile}
      onSelectBlock={async (block) => {
        viewMode = 'notes';
        await selectFile(block.file_path);
      }}
      onRename={doRename}
      onRenameClick={(path) => {
        renamingPath = path;
        renameValue = fileName(path);
        contextMenu = null;
      }}
      onCreateNote={() => {
        createNote();
        contextMenu = null;
      }}
      onContextMenu={(e) => {
        e.preventDefault();
        const target = e.target as HTMLElement;
        const path = target.closest('[data-path]')?.getAttribute('data-path');
        contextMenu = { x: e.clientX, y: e.clientY, path: path ?? null };
      }}
      onViewModeChange={(mode) => {
        viewMode = mode;
        if (mode === 'blocks') loadBlocks();
        if (mode === 'links') loadLinks();
      }}
      onBlockTypeFilterChange={loadBlocks}
      onRenameBlur={(path) => {
        if (renameValue.trim()) doRename(path, renameValue);
        renamingPath = null;
      }}
      onContextMenuClose={() => (contextMenu = null)}
      {fileName}
    >
      {#snippet header()}
        <div class="vault-dropdown">
          <button
            class="open-btn"
            onclick={() => (vaultMenuOpen = !vaultMenuOpen)}
            disabled={loading}
          >
            {vaultPath ? 'Change vault' : 'Open vault'}
            <span class="chevron" class:open={vaultMenuOpen}>▾</span>
          </button>
          {#if vaultMenuOpen}
            <div
              class="vault-menu-backdrop"
              role="button"
              tabindex="-1"
              onclick={() => (vaultMenuOpen = false)}
              onkeydown={(e) => e.key === 'Escape' && (vaultMenuOpen = false)}
            ></div>
            <div class="vault-menu">
              {#if recentVaults.length > 0}
                <div class="vault-menu-label">Recent</div>
                {#each recentVaults as path}
                  <button
                    class="vault-menu-item"
                    onclick={() => openVaultByPath(path)}
                    title={path}
                  >
                    {path.replace(/\\/g, '/').split('/').pop() ?? path}
                  </button>
                {/each}
                <div class="vault-menu-divider"></div>
              {/if}
              <button class="vault-menu-item" onclick={openVault}>
                Open folder…
              </button>
            </div>
          {/if}
        </div>
      {/snippet}
      {#snippet syncRow()}
        <input
          type="text"
          class="sync-input"
          placeholder="Sync: ws://localhost:8765"
          bind:value={syncUrl}
          onblur={() => setSyncUrl(syncUrl)}
        />
      {/snippet}
    </Sidebar>
  {/snippet}
  {#snippet main()}
    {#if viewMode === 'links'}
      <LinksView
        {links}
        focusPath={selectedPath}
        {loading}
        onLinkClick={(pathOrTarget) => {
          const resolved = resolveTargetToPath(pathOrTarget) ?? pathOrTarget;
          viewMode = 'notes';
          selectFile(resolved);
        }}
        {fileName}
      />
    {:else if viewMode === 'blocks'}
      <BlocksView
        {blocks}
        {blockTypeFilter}
        {loading}
        onBlockClick={async (block) => {
          viewMode = 'notes';
          await selectFile(block.file_path);
        }}
        onBlockTypeFilterChange={loadBlocks}
        {fileName}
      />
    {:else if selectedPath}
      <EditorPane
        {editorMode}
        {selectedPath}
        onModeChange={(m) => (editorMode = m)}
        onSaveBeforeModeChange={saveBeforeModeChange}
        getEditorContent={() => editorRef?.getContent?.() ?? editorContent}
      >
        {#snippet children()}
          {#if syncUrl.trim() && syncDoc && syncProvider}
            <EditorSync
              bind:this={editorRef}
              ydoc={syncDoc}
              provider={syncProvider}
              initialContent={content}
              onSave={saveFile}
              onContentChange={(v) => (editorContent = v)}
              debounceMs={2000}
              mode={editorMode}
              completionFiles={files.map((f) => docPathFromVault(f))}
            />
          {:else}
            <Editor
              bind:this={editorRef}
              content={content}
              onSave={saveFile}
              onContentChange={(v) => (editorContent = v)}
              debounceMs={2000}
              mode={editorMode}
              completionFiles={files.map((f) => docPathFromVault(f))}
            />
          {/if}
        {/snippet}
      </EditorPane>
    {:else}
      <Placeholder message={vaultPath ? 'Select a note' : 'Open a vault to get started'} />
    {/if}
  {/snippet}
</AppLayout>

<style>
  .vault-dropdown {
    position: relative;
    min-width: 0;
    flex-shrink: 1;
  }

  .open-btn {
    padding: 6px 12px;
    font-size: 13px;
    background: #2a2a2a;
    color: #e2e8f0;
    border: 1px solid #3a3a3a;
    border-radius: 6px;
    cursor: pointer;
  }

  .open-btn:hover:not(:disabled) {
    background: #333;
  }

  .open-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .open-btn .chevron {
    margin-left: 4px;
    font-size: 10px;
    opacity: 0.7;
    transition: transform 0.15s ease;
  }

  .open-btn .chevron.open {
    transform: rotate(180deg);
  }

  .vault-menu-backdrop {
    position: fixed;
    inset: 0;
    z-index: 100;
  }

  .vault-menu {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 4px;
    min-width: 180px;
    max-width: 240px;
    padding: 4px 0;
    background: #1a1a1a;
    border: 1px solid #2a2a2a;
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    z-index: 101;
  }

  .vault-menu-label {
    padding: 6px 12px;
    font-size: 11px;
    color: #71717a;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .vault-menu-item {
    display: block;
    width: 100%;
    padding: 8px 12px;
    text-align: left;
    font-size: 13px;
    color: #e2e8f0;
    background: none;
    border: none;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .vault-menu-item:hover {
    background: #252525;
  }

  .vault-menu-divider {
    height: 1px;
    margin: 4px 0;
    background: #2a2a2a;
  }

  .sync-input {
    width: 100%;
    min-width: 0;
    box-sizing: border-box;
    padding: 6px 8px;
    font-size: 12px;
    background: #252525;
    color: #a1a1aa;
    border: 1px solid #3a3a3a;
    border-radius: 4px;
  }

  .sync-input::placeholder {
    color: #52525b;
  }
</style>
