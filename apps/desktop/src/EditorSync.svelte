<script lang="ts">
  /**
   * Editor with Yjs sync. Uses y-codemirror.next for real-time collaboration.
   * When sync is enabled, content lives in Y.Text and syncs via WebSocket.
   */
  import { onMount, onDestroy } from 'svelte';
  import { EditorView, basicSetup } from 'codemirror';
  import { EditorState, Compartment } from '@codemirror/state';
  import { markdown } from '@codemirror/lang-markdown';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { marked } from 'marked';
  import { livePreview } from '@nisaba/ui';
  import { yCollab } from 'y-codemirror.next';
  import * as Y from 'yjs';

  let editorContainer: HTMLDivElement;
  let view: EditorView | null = null;

  let {
    ydoc,
    provider,
    initialContent = '',
    onSave = () => {},
    onContentChange = () => {},
    debounceMs = 2000,
    mode = 'edit',
  } = $props<{
    ydoc: Y.Doc;
    provider: { awareness: unknown };
    initialContent?: string;
    onSave?: (value: string) => void;
    onContentChange?: (value: string) => void;
    debounceMs?: number;
    mode?: 'raw' | 'edit' | 'split' | 'read';
  }>();

  let saveTimeout: ReturnType<typeof setTimeout> | null = null;
  const ytext = ydoc.getText('content');

  // Seed Y.Text with initial content if empty (e.g. from local file)
  if (ytext.length === 0 && initialContent) {
    ydoc.transact(() => {
      ytext.insert(0, initialContent);
    });
  }

  function scheduleSave(value: string) {
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => {
      onSave(value);
      saveTimeout = null;
    }, debounceMs);
  }

  let liveContent = $state(ytext.toString());
  const livePreviewCompartment = new Compartment();

  onMount(() => {
    const isRaw = mode === 'raw';
    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        const value = view?.state.doc.toString() ?? '';
        liveContent = value;
        onContentChange(value);
        scheduleSave(value);
      }
    });

    const yCollabExt = yCollab(ytext, provider.awareness, { undoManager: false });
    const extensions = isRaw
      ? [basicSetup, oneDark, yCollabExt, updateListener]
      : [
          basicSetup,
          markdown(),
          oneDark,
          livePreviewCompartment.of(mode === 'edit' ? livePreview : []),
          yCollabExt,
          updateListener,
        ];

    view = new EditorView({
      state: EditorState.create({
        doc: ytext.toString(),
        extensions,
      }),
      parent: editorContainer,
    });

  });

  onDestroy(() => {
    if (saveTimeout) clearTimeout(saveTimeout);
    view?.destroy();
    view = null;
  });

  export function getContent(): string {
    return ytext.toString();
  }

  const renderedHtml = $derived(marked.parse(mode === 'read' ? liveContent : liveContent));
</script>

<div class="editor-wrapper" data-mode={mode}>
  <div
    class="split"
    class:raw-only={mode === 'raw'}
    class:edit-inline={mode === 'edit'}
    class:split-mode={mode === 'split'}
    class:read-only={mode === 'read'}
  >
    <div class="editor-pane" bind:this={editorContainer}></div>
    <div class="preview-pane">
      {@html renderedHtml}
    </div>
  </div>
</div>

<style>
  .editor-wrapper {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }

  .split {
    flex: 1;
    display: flex;
    min-height: 0;
    overflow: hidden;
  }

  .split.raw-only .preview-pane,
  .split.edit-inline .preview-pane {
    display: none;
  }

  .split.raw-only .editor-pane,
  .split.edit-inline .editor-pane {
    flex: 1;
  }

  .split.split-mode .editor-pane,
  .split.split-mode .preview-pane {
    flex: 1;
    min-width: 0;
  }

  .split.split-mode .editor-pane {
    border-right: 1px solid #2a2a2a;
  }

  .split.read-only .editor-pane {
    display: none;
  }

  .split.read-only .preview-pane {
    flex: 1;
  }

  .editor-pane {
    min-width: 0;
    overflow: auto;
  }

  .editor-pane :global(.cm-editor) {
    height: 100%;
    font-size: 14px;
  }

  .editor-pane :global(.cm-scroller) {
    font-family: ui-monospace, 'Cascadia Code', 'Source Code Pro', Menlo, monospace;
  }

  .preview-pane {
    min-width: 0;
    overflow: auto;
    padding: 20px;
    font-size: 14px;
    line-height: 1.6;
    color: #e2e8f0;
  }

  .preview-pane :global(h1) { font-size: 1.5em; margin: 0.5em 0; }
  .preview-pane :global(h2) { font-size: 1.25em; margin: 0.5em 0; }
  .preview-pane :global(h3) { font-size: 1.1em; margin: 0.5em 0; }
  .preview-pane :global(p) { margin: 0.5em 0; }
  .preview-pane :global(ul), .preview-pane :global(ol) { margin: 0.5em 0; padding-left: 1.5em; }
  .preview-pane :global(blockquote) { border-left: 4px solid #3a3a3a; margin: 0.5em 0; padding-left: 1em; color: #a1a1aa; }
  .preview-pane :global(code) { background: #252525; padding: 2px 6px; border-radius: 4px; font-size: 0.9em; }
  .preview-pane :global(pre) { background: #1a1a1a; padding: 12px; border-radius: 6px; overflow-x: auto; }
  .preview-pane :global(pre code) { background: none; padding: 0; }
  .preview-pane :global(a) { color: #6c8eff; }
</style>
