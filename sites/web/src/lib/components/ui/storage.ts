// Device conveniences (theme, list view, collapsed panels) live in localStorage.
// Storage can be unavailable (private windows, blocked site data), so every
// access is guarded and callers fall back to their defaults.

export function readLocal(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

export function writeLocal(key: string, value: string | null): boolean {
  try {
    if (value === null) localStorage.removeItem(key);
    else localStorage.setItem(key, value);
    return true;
  } catch {
    return false;
  }
}
