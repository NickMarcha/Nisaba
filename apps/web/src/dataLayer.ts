/**
 * Web data layer: REST API to Nisaba sync server.
 * Server URL: e.g. http://localhost:8765 (or ws:// for sync).
 */

const SERVER_URL_KEY = 'nisaba-web-server-url';

export function getServerUrl(): string {
  return localStorage.getItem(SERVER_URL_KEY) ?? 'http://localhost:8765';
}

export function setServerUrl(url: string): void {
  if (url.trim()) {
    localStorage.setItem(SERVER_URL_KEY, url.trim());
  } else {
    localStorage.removeItem(SERVER_URL_KEY);
  }
}

function apiBase(): string {
  const u = getServerUrl().trim().replace(/\/$/, '');
  return u.startsWith('ws') ? u.replace(/^ws/, 'http') : u;
}

export async function listFiles(): Promise<string[]> {
  const res = await fetch(`${apiBase()}/api/files`);
  if (!res.ok) throw new Error(`listFiles: ${res.status}`);
  return res.json();
}

export async function readFile(docId: string): Promise<string> {
  const encoded = encodeURIComponent(docId);
  const res = await fetch(`${apiBase()}/api/files/${encoded}`);
  if (!res.ok) {
    if (res.status === 404) return '';
    throw new Error(`readFile: ${res.status}`);
  }
  const json = await res.json();
  return (json.content as string) ?? '';
}

export async function saveFile(docId: string, content: string): Promise<void> {
  const encoded = encodeURIComponent(docId);
  const res = await fetch(`${apiBase()}/api/files/${encoded}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ content }),
  });
  if (!res.ok) throw new Error(`saveFile: ${res.status}`);
}
