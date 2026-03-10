/**
 * Persist last vault and recent vaults for quick reopen.
 */

const LAST_VAULT_KEY = 'nisaba-last-vault';
const RECENT_VAULTS_KEY = 'nisaba-recent-vaults';
const MAX_RECENT = 8;

export function getLastVault(): string | null {
  return localStorage.getItem(LAST_VAULT_KEY);
}

export function getRecentVaults(): string[] {
  try {
    const raw = localStorage.getItem(RECENT_VAULTS_KEY);
    if (!raw) return [];
    const arr = JSON.parse(raw) as string[];
    return Array.isArray(arr) ? arr : [];
  } catch {
    return [];
  }
}

export function setLastVault(path: string): void {
  if (!path.trim()) return;
  localStorage.setItem(LAST_VAULT_KEY, path.trim());

  const recent = getRecentVaults();
  const normalized = path.trim().replace(/\\/g, '/').replace(/\/$/, '');
  const filtered = recent.filter((p) => p.replace(/\\/g, '/').replace(/\/$/, '') !== normalized);
  const updated = [path.trim(), ...filtered].slice(0, MAX_RECENT);
  localStorage.setItem(RECENT_VAULTS_KEY, JSON.stringify(updated));
}

export function removeRecentVault(path: string): void {
  const recent = getRecentVaults();
  const normalized = path.replace(/\\/g, '/').replace(/\/$/, '');
  const filtered = recent.filter((p) => p.replace(/\\/g, '/').replace(/\/$/, '') !== normalized);
  localStorage.setItem(RECENT_VAULTS_KEY, JSON.stringify(filtered));
}
