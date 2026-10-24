/* oxlint-disable eslint/max-classes-per-file -- the test doubles belong together */
// Test doubles for the stores: a clock whose timers run only when told to, a network switch, sessionStorage in a
// Map, and a Personal wired to the mock API (dev/mock-server.ts) through a real ApiClient. Only tests import this.
import { createMockServer, type MockServer } from "../../../dev/mock-server.ts";
import { ApiClient } from "../api/client.ts";
import { createApi, type Api } from "../api/endpoints.ts";
import { indexTerm } from "../core/data.ts";
import type { TermMeta } from "../core/types.ts";
import { plainCell } from "../data/cell.ts";
import { Session, type SessionEnvironment } from "../data/session.ts";
import { DevicePrefs } from "./device.ts";
import type { StoreEnv } from "./env.ts";
import { Personal } from "./personal.ts";

export class MemoryStorage implements Storage {
  readonly #items = new Map<string, string>();

  get length(): number {
    return this.#items.size;
  }

  clear(): void {
    this.#items.clear();
  }

  getItem(key: string): string | null {
    return this.#items.get(key) ?? null;
  }

  key(index: number): string | null {
    return [...this.#items.keys()][index] ?? null;
  }

  removeItem(key: string): void {
    this.#items.delete(key);
  }

  setItem(key: string, value: string): void {
    this.#items.set(key, value);
  }

  /** Every key, for assertions. */
  keys(): string[] {
    return [...this.#items.keys()];
  }
}

/** Lets pending promise callbacks (and the mock server's async work) run. */
export async function settle(): Promise<void> {
  for (let i = 0; i < 20; i++) {
    // oxlint-disable-next-line no-await-in-loop -- each turn lets the next round of callbacks run
    await new Promise<void>((resolve) => {
      setTimeout(resolve, 0);
    });
  }
}

export class FakeEnv implements StoreEnv {
  time = 1_000_000;
  isOnline = true;
  readonly store = new MemoryStorage();
  readonly #timers = new Map<number, { at: number; fn: () => void }>();
  readonly #online = new Set<() => void>();
  #next = 1;

  now(): number {
    return this.time;
  }

  setTimeout(fn: () => void, ms: number): unknown {
    const id = this.#next++;
    this.#timers.set(id, { at: this.time + ms, fn });
    return id;
  }

  clearTimeout(handle: unknown): void {
    if (typeof handle === "number") this.#timers.delete(handle);
  }

  online(): boolean {
    return this.isOnline;
  }

  onOnline(fn: () => void): () => void {
    this.#online.add(fn);
    return () => {
      this.#online.delete(fn);
    };
  }

  storage(): Storage {
    return this.store;
  }

  /** Timers waiting to run. */
  get pending(): number {
    return this.#timers.size;
  }

  /** Moves the clock forward, running every timer that comes due (and what they start) on the way. */
  async advance(ms: number): Promise<void> {
    const target = this.time + ms;
    for (;;) {
      // oxlint-disable-next-line no-await-in-loop -- timers run in order, each after the last one's work settles
      await settle();
      const due = [...this.#timers]
        .filter(([, t]) => t.at <= target)
        .toSorted((a, b) => a[1].at - b[1].at)[0];
      if (due === undefined) break;
      this.#timers.delete(due[0]);
      this.time = Math.max(this.time, due[1].at);
      due[1].fn();
    }
    this.time = target;
    await settle();
  }

  /** Forgets every timer and listener: what leaving the page does to the old page's. */
  dropTimers(): void {
    this.#timers.clear();
    this.#online.clear();
  }

  /** The network comes back. */
  async goOnline(): Promise<void> {
    this.isOnline = true;
    for (const fn of this.#online) fn();
    await settle();
  }
}

export interface Harness {
  env: FakeEnv;
  server: MockServer;
  api: Api;
  session: Session;
  personal: Personal;
  navigations: string[];
  /** Same server and storage, new everything else: what a page reload does. */
  reload(): Promise<Harness>;
}

export const DEBOUNCE_MS = 1000;

const S27: TermMeta = {
  id: "S27",
  name: "Spring 2027",
  season: "spring",
  year: 2027,
  minis: [],
  classesBegin: "2027-01-19",
  classesEnd: "2027-04-30",
  noClasses: [],
};

/** A signed-out (or signed-in) website against a fresh mock API, attached and settled. */
export async function harness(
  options: { signedIn?: boolean; env?: FakeEnv; server?: MockServer; path?: string } = {},
): Promise<Harness> {
  const env = options.env ?? new FakeEnv();
  const server =
    options.server ??
    createMockServer({ catalogDir: null, signedIn: options.signedIn, now: () => env.now() });
  const client = new ApiClient({ fetch: server.fetch });
  const api = createApi(client);
  const navigations: string[] = [];
  const sessionEnv: SessionEnvironment = {
    currentPath: () => options.path ?? "/schedules?term=S27",
    navigate: (url) => {
      navigations.push(url);
    },
    storage: () => env.store,
  };
  const session = new Session(api, plainCell, sessionEnv);
  client.onUnauthorized((path) => {
    if (path !== "/api/me") session.expired();
  });
  const personal = new Personal({
    api,
    session,
    device: new DevicePrefs(plainCell, () => null),
    cell: plainCell,
    env,
    debounceMs: DEBOUNCE_MS,
    loadTerm: () => Promise.resolve(indexTerm({ term: S27, courses: [] })),
  });
  personal.attach();
  await session.refresh();
  await settle();
  const h: Harness = {
    env,
    server,
    api,
    session,
    personal,
    navigations,
    reload: () => {
      env.dropTimers();
      return harness({ ...options, env, server, signedIn: server.signedIn });
    },
  };
  return h;
}

/** Signed-out stores with no server behind them, ready at once: for tests of the stores' own edits. */
export function draftStores(env = new FakeEnv()): Personal {
  const api = createApi(
    new ApiClient({ fetch: () => Promise.reject(new TypeError("no network in this test")) }),
  );
  const session = new Session(api, plainCell, {
    currentPath: () => "/",
    navigate: () => {
      // Tests don't leave the page.
    },
    storage: () => env.store,
  });
  const personal = new Personal({
    api,
    session,
    device: new DevicePrefs(plainCell, () => null),
    cell: plainCell,
    env,
    loadTerm: () => Promise.reject(new Error("no catalog in this test")),
  });
  personal.attach();
  return personal;
}
