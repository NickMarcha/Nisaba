/**
 * Sync layer: connect to Nisaba sync server via y-websocket.
 * Server expects: ws://host:port/sync/{doc_id}
 */

import * as Y from 'yjs';
import { WebsocketProvider } from 'y-websocket';
import { getServerUrl } from './dataLayer';

export const META_DOC_ID = '_meta';
export const BLOCKS_DOC_ID = '_blocks';

function buildSyncUrl(): string {
  const u = getServerUrl().trim().replace(/\/$/, '');
  const base = u.startsWith('ws') ? u : u.replace(/^http/, 'ws');
  return base.endsWith('/sync') ? base : `${base}/sync`;
}

/**
 * docId: doc path, e.g. "notes/foo.md"
 */
export function connectSync(docId: string, ydoc: Y.Doc): WebsocketProvider {
  const serverUrl = buildSyncUrl();
  const roomname = encodeURIComponent(docId.replace(/\\/g, '/'));
  return new WebsocketProvider(serverUrl, roomname, ydoc);
}

/**
 * Return a promise that resolves when the provider is synced.
 * y-websocket v3 uses provider.synced and 'sync' event instead of whenSynced.
 */
export function whenSynced(provider: WebsocketProvider): Promise<void> {
  const p = provider as { whenSynced?: Promise<void> };
  if (p.whenSynced) return p.whenSynced;
  if (provider.synced) return Promise.resolve();
  return new Promise((resolve) => {
    const onSync = (isSynced: boolean) => {
      if (isSynced) {
        if ('off' in provider && typeof provider.off === 'function') {
          provider.off('sync', onSync);
        }
        resolve();
      }
    };
    provider.on('sync', onSync);
  });
}

/**
 * Subscribe to _meta (file list) via WebSocket. Returns files array from Y.Text content (JSON).
 */
export function connectMeta(
  onFiles: (files: string[]) => void
): { provider: WebsocketProvider; ydoc: Y.Doc } {
  const ydoc = new Y.Doc();
  const provider = connectSync(META_DOC_ID, ydoc);
  const text = ydoc.getText('content');
  const handler = () => {
    try {
      const raw = text.toString();
      const parsed = raw ? (JSON.parse(raw) as string[]) : [];
      onFiles(Array.isArray(parsed) ? parsed : []);
    } catch {
      onFiles([]);
    }
  };
  text.observe(handler);
  whenSynced(provider).then(handler);
  return { provider, ydoc };
}

/**
 * Subscribe to _blocks (blocks + links index) via WebSocket.
 */
export function connectBlocks(
  onIndex: (blocks: IndexedBlock[], links: IndexedLink[]) => void
): { provider: WebsocketProvider; ydoc: Y.Doc } {
  const ydoc = new Y.Doc();
  const provider = connectSync(BLOCKS_DOC_ID, ydoc);
  const text = ydoc.getText('content');
  const handler = () => {
    try {
      const raw = text.toString();
      const data = raw ? (JSON.parse(raw) as { blocks?: IndexedBlock[]; links?: IndexedLink[] }) : {};
      onIndex(data.blocks ?? [], data.links ?? []);
    } catch {
      onIndex([], []);
    }
  };
  text.observe(handler);
  whenSynced(provider).then(handler);
  return { provider, ydoc };
}

export interface IndexedBlock {
  id: string;
  file_path: string;
  block_index: number;
  block_type: string | null;
  content: string;
}

export interface IndexedLink {
  source_file: string;
  target: string;
  relation_key: string | null;
}
