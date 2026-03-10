<script lang="ts">
  import * as Y from 'yjs';
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
  import {
    getServerUrl,
    setServerUrl,
    saveFile,
  } from './dataLayer';
  import { connectSync, connectMeta, connectBlocks, whenSynced } from './sync';

  let serverUrl = $state(getServerUrl());
  let files = $state<string[]>([]);
  let blocks = $state<IndexedBlock[]>([]);
  let links = $state<IndexedLink[]>([]);
  let selectedPath = $state<string | null>(null);
  let content = $state('');
  let editorContent = $state('');
  let loading = $state(false);
  let error = $state<string | null>(null);
  let viewMode = $state<'notes' | 'blocks' | 'links'>('notes');
  let blockTypeFilter = $state('');
  let editorMode = $state<'raw' | 'edit' | 'split' | 'read'>('edit');
  let editorRef = $state<Editor | InstanceType<typeof EditorSync> | null>(null);
  let contextMenu = $state<{ x: number; y: number; path: string | null } | null>(null);
  let syncDoc = $state<Y.Doc | null>(null);
  let syncProvider = $state<ReturnType<typeof connectSync> | null>(null);
  let metaProvider = $state<ReturnType<typeof connectMeta>['provider'] | null>(null);
  let metaYdoc = $state<Y.Doc | null>(null);
  let blocksProvider = $state<ReturnType<typeof connectBlocks>['provider'] | null>(null);
  let blocksYdoc = $state<Y.Doc | null>(null);

  $effect(() => {
    if (!serverUrl.trim()) disconnect();
  });

  async function connect() {
    if (!serverUrl.trim()) return;
    loading = true;
    error = null;
    try {
      setServerUrl(serverUrl);
      metaProvider?.destroy();
      metaYdoc?.destroy();
      blocksProvider?.destroy();
      blocksYdoc?.destroy();
      const { provider: metaProv, ydoc: metaY } = connectMeta((f) => (files = f));
      metaProvider = metaProv;
      metaYdoc = metaY;
      await whenSynced(metaProv);
      const { provider: blocksProv, ydoc: blocksY } = connectBlocks((b, l) => {
        blocks = b;
        links = l;
      });
      blocksProvider = blocksProv;
      blocksYdoc = blocksY;
      await whenSynced(blocksProv);
      selectedPath = null;
      content = '';
    } catch (e) {
      error = String(e);
      files = [];
      blocks = [];
      links = [];
    } finally {
      loading = false;
    }
  }

  function disconnect() {
    metaProvider?.destroy();
    metaProvider = null;
    metaYdoc?.destroy();
    metaYdoc = null;
    blocksProvider?.destroy();
    blocksProvider = null;
    blocksYdoc?.destroy();
    blocksYdoc = null;
    syncProvider?.destroy();
    syncProvider = null;
    syncDoc?.destroy();
    syncDoc = null;
    files = [];
    blocks = [];
    links = [];
    selectedPath = null;
    content = '';
  }

  async function selectFile(path: string) {
    if (selectedPath) {
      const currentContent = editorRef?.getContent?.() ?? editorContent;
      if (currentContent !== content) {
        await saveFile(selectedPath, currentContent);
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
    selectedPath = path;
    content = '';
    editorContent = '';
    try {
      // Connect directly to sync; server sends state on connect. Avoids REST + sync
      // both loading content (which caused 3–4 visible updates on first open).
      const ydoc = new Y.Doc();
      const provider = connectSync(path, ydoc);
      syncDoc = ydoc;
      syncProvider = provider;
      // Wait for initial sync so we have content before showing editor
      await whenSynced(provider);
      content = ydoc.getText('content').toString();
      editorContent = content;
    } catch (e) {
      error = String(e);
      content = '';
      editorContent = '';
    } finally {
      loading = false;
    }
  }

  async function saveFileHandler(value: string) {
    if (!selectedPath) return;
    content = value;
    editorContent = value;
    // When sync is active, server receives updates via WebSocket; no need to PUT.
    if (syncProvider) return;
    try {
      await saveFile(selectedPath, value);
      error = null;
    } catch (e) {
      error = String(e);
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
    return path.replace(/\\/g, '/').split('/').pop() ?? path;
  }

  async function createNote() {
    if (!serverUrl.trim()) return;
    const exists = (name: string) => files.includes(name) || files.some((f) => f.endsWith('/' + name));
    let name = 'untitled.md';
    let n = 1;
    while (exists(name)) {
      name = `untitled-${n}.md`;
      n++;
    }
    try {
      await saveFile(name, '# Untitled\n\n');
      if (!files.includes(name)) files = [...files, name].sort();
      await selectFile(name);
    } catch (e) {
      error = String(e);
    }
  }

  async function saveBeforeModeChange() {
    const c = editorRef?.getContent?.() ?? editorContent;
    if (c !== content && selectedPath) await saveFileHandler(c);
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
      renamingPath={null}
      renameValue=""
      {contextMenu}
      treeView={true}
      pathToRelative={(p) => p}
      onSelectFile={selectFile}
      onSelectBlock={(block) => {
        viewMode = 'notes';
        selectFile(block.file_path);
      }}
      onRename={() => {}}
      onRenameClick={() => {}}
      onCreateNote={createNote}
      onContextMenu={(e) => {
        e.preventDefault();
        const target = e.target as HTMLElement;
        const path = target.closest('[data-path]')?.getAttribute('data-path');
        contextMenu = { x: e.clientX, y: e.clientY, path: path ?? null };
      }}
      onViewModeChange={(m) => (viewMode = m)}
      onBlockTypeFilterChange={() => {}}
      onRenameBlur={() => {}}
      onContextMenuClose={() => (contextMenu = null)}
      {fileName}
    >
      {#snippet syncRow()}
        <div class="server-row">
          <input
            type="text"
            class="server-input"
            placeholder="Server: http://localhost:8765"
            bind:value={serverUrl}
            onblur={() => setServerUrl(serverUrl)}
          />
          <button class="connect-btn" onclick={connect} disabled={loading}>
            Connect
          </button>
        </div>
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
        blocks={blockTypeFilter ? blocks.filter((b) => b.block_type === blockTypeFilter) : blocks}
        {blockTypeFilter}
        {loading}
        onBlockClick={(block) => {
          viewMode = 'notes';
          selectFile(block.file_path);
        }}
        onBlockTypeFilterChange={() => {}}
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
          {#if syncDoc && syncProvider}
            <EditorSync
              bind:this={editorRef}
              ydoc={syncDoc}
              provider={syncProvider}
              initialContent={content}
              onSave={saveFileHandler}
              onContentChange={(v) => (editorContent = v)}
              debounceMs={2000}
              mode={editorMode}
              completionFiles={files}
            />
          {:else}
            <Editor
              bind:this={editorRef}
              content={content}
              onSave={saveFileHandler}
              onContentChange={(v) => (editorContent = v)}
              debounceMs={2000}
              mode={editorMode}
              completionFiles={files}
            />
          {/if}
        {/snippet}
      </EditorPane>
    {:else}
      <Placeholder
        message={serverUrl.trim() && files.length > 0 ? 'Select a note or create one' : 'Enter server URL and click Connect'}
      />
    {/if}
  {/snippet}
</AppLayout>

<style>
  .server-row {
    display: flex;
    gap: 8px;
    align-items: center;
    min-width: 0;
  }

  .server-input {
    flex: 1;
    min-width: 0;
    padding: 6px 8px;
    font-size: 12px;
    background: #252525;
    color: #a1a1aa;
    border: 1px solid #3a3a3a;
    border-radius: 4px;
  }

  .server-input::placeholder {
    color: #52525b;
  }

  .connect-btn {
    padding: 6px 12px;
    font-size: 13px;
    background: #2a2a2a;
    color: #e2e8f0;
    border: 1px solid #3a3a3a;
    border-radius: 6px;
    cursor: pointer;
  }

  .connect-btn:hover:not(:disabled) {
    background: #333;
  }

  .connect-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
