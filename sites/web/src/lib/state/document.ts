// One record per student (the academic record, preferences), held in memory and kept on the server like the
// collections in collection.ts: a sessionStorage draft when signed out, saved with revisions when signed in, and
// whatever the server doesn't have yet kept in sessionStorage until it does.
import type { Cell } from "../data/cell.ts";
import {
  DEFAULT_DEBOUNCE_MS,
  draftKey,
  localCopy,
  parseStored,
  pendingKey,
  readCopy,
  SESSION_ENDED,
  type LocalCopy,
  type StoreDeps,
  type StoreMode,
} from "./drafts.ts";
import { writeJson, type StoreEnv } from "./env.ts";
import { RecordSync, type Edit, type SaveStatus, type Stored } from "./sync.ts";

export interface DocumentAdapter<T> {
  /** Names the sessionStorage keys. */
  name: string;
  /** The value before anything is saved. */
  empty(): T;
  /** The stored record; revision 0 when nothing has been saved yet. */
  get(): Promise<Stored<T>>;
  /** Replaces revision `revision` (0 creates). */
  put(value: T, revision: number): Promise<Stored<T>>;
  /** A value from untrusted JSON (a 409's `current`, a stored draft), or null. */
  normalize(raw: unknown): T | null;
  /** This device's value combined with the server's, when the edits that made it aren't known. */
  merge(local: T, remote: T): T;
  /** Whether `value` has anything worth saving (an empty draft isn't imported). */
  isEmpty(value: T): boolean;
}

export class DocumentStore<T> {
  readonly #a: DocumentAdapter<T>;
  readonly #env: StoreEnv;
  readonly #debounceMs: number;
  readonly #value: Cell<T>;
  readonly #mode: Cell<StoreMode>;
  readonly #status: Cell<SaveStatus>;
  readonly #error: Cell<string | null>;
  #sync: RecordSync<T> | null = null;
  #owner: string | null = null;
  #expired = false;
  #epoch = 0;

  constructor(adapter: DocumentAdapter<T>, deps: StoreDeps) {
    this.#a = adapter;
    this.#env = deps.env;
    this.#debounceMs = deps.debounceMs ?? DEFAULT_DEBOUNCE_MS;
    this.#value = deps.cell(adapter.empty());
    this.#mode = deps.cell<StoreMode>("draft");
    this.#status = deps.cell<SaveStatus>("saved");
    this.#error = deps.cell<string | null>(null);
  }

  get #draftKey(): string {
    return draftKey(this.#a.name);
  }

  get #pendingKey(): string {
    return pendingKey(this.#a.name);
  }

  get value(): T {
    return this.#value.value;
  }

  get mode(): StoreMode {
    return this.#mode.value;
  }

  get status(): SaveStatus {
    return this.#status.value;
  }

  get error(): string | null {
    return this.#error.value;
  }

  /** Signed in and the server doesn't have everything yet. */
  get unsaved(): boolean {
    return this.#mode.value === "account" && (this.#sync?.unsaved ?? false);
  }

  /** Applies an edit now; signed in, it's saved after a short pause. Returns the new value. */
  edit(fn: Edit<T>): T {
    const current = this.#value.value;
    const next = this.#sync === null ? fn(current) : this.#sync.edit(fn);
    if (next !== current) {
      this.#value.value = next;
      this.#changed();
    }
    return next;
  }

  #changed(): void {
    this.#persist();
    this.#refreshStatus();
  }

  #refreshStatus(): void {
    if (this.#expired) {
      this.#status.value = "error";
      this.#error.value = SESSION_ENDED;
    } else if (this.#mode.value === "account" && this.#sync !== null) {
      this.#status.value = this.#sync.status;
      this.#error.value = this.#sync.error;
    } else {
      this.#status.value = "saved";
      this.#error.value = null;
    }
  }

  #persist(): void {
    if (this.#mode.value !== "account") {
      const value = this.#value.value;
      writeJson(this.#env, this.#draftKey, this.#a.isEmpty(value) ? null : { value });
      return;
    }
    const sync = this.#sync;
    const pending =
      sync?.unsaved === true
        ? { owner: this.#owner, value: sync.value, basedOn: sync.revision }
        : null;
    writeJson(this.#env, this.#pendingKey, pending);
  }

  #readLocal(key: string, owner: string | null): LocalCopy<T> | null {
    const raw = readCopy(this.#env, key, owner);
    return raw === null ? null : localCopy(raw, (v) => this.#a.normalize(v));
  }

  #newSync(init: {
    value: T;
    revision: number;
    unsaved: boolean;
    editsKnown: boolean;
  }): RecordSync<T> {
    const epoch = this.#epoch;
    return new RecordSync<T>(
      {
        env: this.#env,
        debounceMs: this.#debounceMs,
        save: (value, revision) => this.#a.put(value, revision ?? 0),
        parse: (current) => parseStored(current, (raw) => this.#a.normalize(raw)),
        merge: (local, remote) => this.#a.merge(local, remote),
        onValue: (value) => {
          if (epoch === this.#epoch) this.#value.value = value;
        },
        onStatus: () => {
          if (epoch === this.#epoch) this.#changed();
        },
        recreateWhenMissing: false,
      },
      init,
    );
  }

  #reset(mode: StoreMode): void {
    this.#epoch += 1;
    this.#sync?.stop();
    this.#sync = null;
    this.#expired = false;
    this.#mode.value = mode;
  }

  /** Signed out: shows this tab's draft. */
  startDraft(): void {
    this.#reset("draft");
    this.#owner = null;
    this.#value.value = this.#readLocal(this.#draftKey, null)?.value ?? this.#a.empty();
    this.#refreshStatus();
  }

  /**
   * Signed in as `owner`: loads the stored record, then folds in this tab's draft and anything an earlier visit
   * didn't get to save. Rejects when it can't be loaded.
   */
  async startAccount(owner: string): Promise<void> {
    if (this.#mode.value === "account") this.#value.value = this.#a.empty();
    this.#reset("loading");
    this.#owner = owner;
    this.#refreshStatus();
    const epoch = this.#epoch;
    const stored = await this.#a.get();
    if (epoch !== this.#epoch) return;

    let value = stored.value;
    let editsKnown = true;
    const pending = this.#readLocal(this.#pendingKey, owner);
    if (pending !== null) {
      value =
        pending.basedOn === stored.revision ? pending.value : this.#a.merge(pending.value, value);
      editsKnown = false;
    }
    const draft = this.#readLocal(this.#draftKey, null);
    if (draft !== null && !this.#a.isEmpty(draft.value)) {
      value = this.#a.merge(draft.value, value);
      editsKnown = false;
    }
    this.#mode.value = "account";
    const unsaved = value !== stored.value;
    this.#sync = this.#newSync({
      value,
      revision: stored.revision,
      unsaved,
      editsKnown: editsKnown || !unsaved,
    });
    this.#value.value = value;
    writeJson(this.#env, this.#draftKey, null);
    this.#changed();
  }

  /** The session ended unexpectedly: stops writing, keeps the value on screen and the unsaved part in storage. */
  expire(): void {
    if (this.#mode.value !== "account") return;
    this.#persist();
    this.#epoch += 1;
    this.#sync?.stop();
    this.#expired = true;
    this.#refreshStatus();
  }

  /** Signed out on purpose: forgets the account's record and anything unsaved, and starts an empty draft. */
  clear(): void {
    this.#reset("draft");
    writeJson(this.#env, this.#pendingKey, null);
    writeJson(this.#env, this.#draftKey, null);
    this.#owner = null;
    this.#value.value = this.#a.empty();
    this.#refreshStatus();
  }

  retry(): void {
    if (this.#mode.value === "account" && !this.#expired) this.#sync?.retry();
  }

  async flush(): Promise<void> {
    if (this.#mode.value === "account" && !this.#expired) await this.#sync?.flush();
  }
}
