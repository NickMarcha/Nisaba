<script lang="ts">
  import type { EditorMode } from './types';

  let {
    editorMode = 'edit',
    selectedPath,
    onModeChange,
    onSaveBeforeModeChange,
    getEditorContent,
    children,
  } = $props<{
    editorMode?: EditorMode;
    selectedPath?: string | null;
    onModeChange?: (mode: EditorMode) => void;
    onSaveBeforeModeChange?: () => Promise<void>;
    getEditorContent?: () => string;
    children: import('svelte').Snippet;
  }>();

  async function handleModeChange(mode: EditorMode) {
    const content = getEditorContent?.() ?? '';
    await onSaveBeforeModeChange?.();
    onModeChange?.(mode);
  }
</script>

<div class="editor-pane">
  <div class="editor-toolbar">
    <button
      class="mode-btn"
      class:active={editorMode === 'raw'}
      onclick={() => handleModeChange('raw')}
    >
      Raw
    </button>
    <button
      class="mode-btn"
      class:active={editorMode === 'edit'}
      onclick={() => handleModeChange('edit')}
    >
      Edit
    </button>
    <button
      class="mode-btn"
      class:active={editorMode === 'split'}
      onclick={() => handleModeChange('split')}
    >
      Split
    </button>
    <button
      class="mode-btn"
      class:active={editorMode === 'read'}
      onclick={() => handleModeChange('read')}
    >
      Read
    </button>
  </div>
  {#key `${editorMode}-${selectedPath ?? ''}`}
    {@render children()}
  {/key}
</div>

<style>
  .editor-pane {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .editor-toolbar {
    display: flex;
    gap: 4px;
    padding: 8px 16px;
    border-bottom: 1px solid #2a2a2a;
  }

  .mode-btn {
    padding: 4px 12px;
    font-size: 12px;
    background: #252525;
    color: #a1a1aa;
    border: 1px solid #2a2a2a;
    border-radius: 4px;
    cursor: pointer;
  }

  .mode-btn:hover {
    background: #2a2a2a;
    color: #f5f5f5;
  }

  .mode-btn.active {
    background: #333;
    color: #f5f5f5;
  }
</style>
