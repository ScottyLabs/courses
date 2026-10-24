// What the stores need from the browser, gathered so tests can supply their own: timers and a clock, whether the
// network is up, and sessionStorage for drafts.

export interface StoreEnv {
  now(): number;
  setTimeout(fn: () => void, ms: number): unknown;
  clearTimeout(handle: unknown): void;
  /** False when the browser knows it's offline. */
  online(): boolean;
  /** Calls `fn` when the browser comes back online. Returns an unsubscribe. */
  onOnline(fn: () => void): () => void;
  /** sessionStorage, or null when it's unavailable (drafts then last only as long as the page). */
  storage(): Storage | null;
}

function noop(): void {
  // Nothing to undo.
}

function sessionStore(): Storage | null {
  try {
    return typeof sessionStorage === "undefined" ? null : sessionStorage;
  } catch {
    return null;
  }
}

export const browserEnv: StoreEnv = {
  now: () => Date.now(),
  setTimeout: (fn, ms) => globalThis.setTimeout(fn, ms),
  clearTimeout: (handle) => {
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- handles only ever come from setTimeout above
    globalThis.clearTimeout(handle as ReturnType<typeof setTimeout>);
  },
  online: () => {
    if (typeof navigator === "undefined") return true;
    // Read without the DOM types, which Deno (running the tests) doesn't have for navigator.
    const online: unknown = Reflect.get(navigator, "onLine");
    return online !== false;
  },
  onOnline: (fn) => {
    if (typeof addEventListener === "undefined") return noop;
    addEventListener("online", fn);
    return () => {
      removeEventListener("online", fn);
    };
  },
  storage: sessionStore,
};

/** Parsed JSON from storage, or null when missing, unreadable or storage is off. */
export function readJson(env: StoreEnv, key: string): unknown {
  try {
    const text = env.storage()?.getItem(key) ?? null;
    return text === null ? null : (JSON.parse(text) as unknown);
  } catch {
    return null;
  }
}

/** Writes JSON to storage; `null` removes the key. Quietly does nothing when storage is off or full. */
export function writeJson(env: StoreEnv, key: string, value: unknown): void {
  try {
    const storage = env.storage();
    if (value === null) storage?.removeItem(key);
    else storage?.setItem(key, JSON.stringify(value));
  } catch {
    // Storage off or full: the change stays on screen; it just won't survive a reload.
  }
}

export const isObject = (v: unknown): v is Record<string, unknown> =>
  typeof v === "object" && v !== null && !Array.isArray(v);
