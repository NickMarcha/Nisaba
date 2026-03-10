/**
 * Shared Svelte components for Nisaba.
 * Used by both desktop (Tauri) and web clients.
 */

export { default as AppLayout } from './AppLayout.svelte';
export { default as Sidebar } from './Sidebar.svelte';
export { default as EditorPane } from './EditorPane.svelte';
export { default as Editor } from './Editor.svelte';
export { default as EditorSync } from './EditorSync.svelte';
export { default as BlocksView } from './BlocksView.svelte';
export { default as LinksView } from './LinksView.svelte';
export { default as Placeholder } from './Placeholder.svelte';
export { livePreview } from './editor/livePreview';
export type { IndexedBlock, IndexedLink, ViewMode, EditorMode } from './types';
