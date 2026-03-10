/**
 * Sync layer: connect to Nisaba sync server via y-websocket.
 * Server expects: ws://host:port/sync/{doc_id}
 * y-websocket builds URL as serverUrl + '/' + roomname, so we use:
 *   serverUrl = base + '/sync', roomname = encodeURIComponent(docPath)
 */

import * as Y from 'yjs';
import { WebsocketProvider } from 'y-websocket';

const SYNC_URL_KEY = 'nisaba-sync-url';

export function getSyncUrl(): string {
  return localStorage.getItem(SYNC_URL_KEY) ?? '';
}

export function setSyncUrl(url: string): void {
  if (url.trim()) {
    localStorage.setItem(SYNC_URL_KEY, url.trim());
  } else {
    localStorage.removeItem(SYNC_URL_KEY);
  }
}

/**
 * Build the server URL for y-websocket. User enters e.g. "ws://localhost:8765".
 * We append /sync so the final path is /sync/{roomname}.
 */
function buildServerUrl(base: string): string {
  const u = base.trim().replace(/\/$/, '');
  return u.endsWith('/sync') ? u : `${u}/sync`;
}

function apiBase(baseUrl: string): string {
  const u = baseUrl.trim().replace(/\/$/, '');
  return u.startsWith('ws') ? u.replace(/^ws/, 'http') : u;
}

/**
 * Fetch doc IDs from server. Returns paths relative to vault (doc_id).
 */
export async function listServerFiles(baseUrl: string): Promise<string[]> {
  const res = await fetch(`${apiBase(baseUrl)}/api/files`);
  if (!res.ok) throw new Error(`listServerFiles: ${res.status}`);
  return res.json();
}

/**
 * Fetch doc content from server. For server-only docs (not yet on disk).
 */
export async function readFileFromServer(baseUrl: string, docId: string): Promise<string> {
  const encoded = encodeURIComponent(docId);
  const res = await fetch(`${apiBase(baseUrl)}/api/files/${encoded}`);
  if (!res.ok) {
    if (res.status === 404) return '';
    throw new Error(`readFileFromServer: ${res.status}`);
  }
  const json = await res.json();
  return (json.content as string) ?? '';
}

/**
 * docPath: path relative to vault root, e.g. "notes/foo.md"
 * Must be URL-safe when used as roomname (slash becomes %2F).
 */
export function connectSync(
  baseUrl: string,
  docPath: string,
  ydoc: Y.Doc
): WebsocketProvider {
  const serverUrl = buildServerUrl(baseUrl);
  const roomname = encodeURIComponent(docPath.replace(/\\/g, '/'));
  return new WebsocketProvider(serverUrl, roomname, ydoc);
}

/**
 * Resolves when the provider has synced with the server.
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
