/* oxlint-disable eslint/max-lines -- one store class; its draft, sync and lifecycle code share private state */
// A set of records the student owns one by one (schedules, degree plans), held in memory for the UI and kept on the
// server (decision 17). Signed out it's a draft in sessionStorage. Signed in, every record saves on its own
// (sync.ts), and whatever the server doesn't have yet is also kept in sessionStorage, so a reload, a lost connection
// or the trip through sign-in loses nothing; those copies are written back when the account loads again.
import { ApiError } from "../api/errors.ts";
import type { Cell } from "../data/cell.ts";
import {
  DEFAULT_DEBOUNCE_MS,
  draftKey,
  parseStored,
  pendingKey,
  readCollectionCopy,
  SESSION_ENDED,
  type LocalCopy,
  type StoreDeps,
  type StoreMode,
} from "./drafts.ts";
import { writeJson, type StoreEnv } from "./env.ts";
import { RecordSync, worstStatus, type Edit, type SaveStatus, type Stored } from "./sync.ts";

/** How a collection reaches the server, and what its records look like. */
export interface CollectionAdapter<T extends { id: string }> {
  /** Names the sessionStorage keys. */
  name: string;
  list(): Promise<Stored<T>[]>;
  create(value: T): Promise<Stored<T>>;
  update(value: T, revision: number): Promise<Stored<T>>;
  remove(id: string): Promise<void>;
  /** A record from untrusted JSON (a 409's `current`, a stored draft), or null. */
  normalize(raw: unknown): T | null;
  /** This device's copy combined with the server's, when the edits that made it aren't known. */
  merge(local: T, remote: T): T;
  /**
   * A signed-out draft record the account doesn't have, joining the account's `existing` records on sign-in: the
   * record to add (renamed so no two share a name) and whether to save it now, or null to leave it out (an empty
   * default the account already has one of). Without it, drafts are added as they are.
   */
  adoptDraft?(value: T, existing: readonly T[]): { value: T; save: boolean } | null;
}

export class CollectionStore<T extends { id: string }> {
  readonly #a: CollectionAdapter<T>;
  readonly #env: StoreEnv;
  readonly #debounceMs: number;
  readonly #items: Cell<Record<string, T>>;
  readonly #mode: Cell<StoreMode>;
  readonly #status: Cell<SaveStatus>;
  readonly #error: Cell<string | null>;
  readonly #syncs = new Map<string, RecordSync<T>>();
  /** Deleted here, maybe still on the server: id to the revision last seen. */
  readonly #deletes = new Map<string, number | null>();
  #deleteStatus: SaveStatus = "saved";
  #deleteError: string | null = null;
  #owner: string | null = null;
  #expired = false;
  /** Bumped on every mode change, so answers from a previous session are ignored. */
  #epoch = 0;

  constructor(adapter: CollectionAdapter<T>, deps: StoreDeps) {
    this.#a = adapter;
    this.#env = deps.env;
    this.#debounceMs = deps.debounceMs ?? DEFAULT_DEBOUNCE_MS;
    this.#items = deps.cell<Record<string, T>>({});
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

  /** Every record, by id. */
  get items(): Record<string, T> {
    return this.#items.value;
  }

  get(id: string): T | undefined {
    return this.#items.value[id];
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
    if (this.#mode.value !== "account") return false;
    return this.#deletes.size > 0 || [...this.#syncs.values()].some((s) => s.unsaved);
  }

  #setItem(id: string, value: T): void {
    this.#items.value = { ...this.#items.value, [id]: value };
  }

  #removeItem(id: string): void {
    const next = { ...this.#items.value };
    delete next[id];
    this.#items.value = next;
  }

  // ---- Edits ----

  /** Adds a record. `save: false` keeps it on this device until its first edit (an empty schedule made on visit). */
  create(value: T, options: { save?: boolean } = {}): void {
    this.#setItem(value.id, value);
    this.#deletes.delete(value.id);
    if (this.#mode.value === "account") {
      this.#syncs.get(value.id)?.stop();
      const unsaved = options.save !== false;
      this.#syncs.set(value.id, this.#sync({ value, revision: null, unsaved, editsKnown: true }));
    }
    this.#changed();
  }

  /** Applies an edit to a record. Returns the new value, or undefined when there's no such record. */
  edit(id: string, fn: Edit<T>): T | undefined {
    const current = this.#items.value[id];
    if (current === undefined) return undefined;
    const sync = this.#syncs.get(id);
    const next = sync === undefined ? fn(current) : sync.edit(fn);
    if (next !== current) {
      this.#setItem(id, next);
      this.#changed();
    }
    return next;
  }

  /** Changes a record without saving it, for fields the server sets itself. */
  amend(id: string, fn: Edit<T>): void {
    const current = this.#items.value[id];
    if (current === undefined) return;
    const sync = this.#syncs.get(id);
    const next = sync === undefined ? fn(current) : sync.amend(fn);
    if (next !== current) this.#setItem(id, next);
  }

  delete(id: string): void {
    if (this.#items.value[id] === undefined) return;
    this.#removeItem(id);
    const sync = this.#syncs.get(id);
    this.#syncs.delete(id);
    if (sync !== undefined && this.#mode.value === "account") {
      sync.stop();
      const epoch = this.#epoch;
      // A record still being created is deleted once the create is through.
      void sync.settled().then(() => {
        if (epoch === this.#epoch && sync.sent) this.#queueDelete(id, sync.revision);
      });
    }
    this.#changed();
  }

  #queueDelete(id: string, revision: number | null): void {
    this.#deletes.set(id, revision);
    this.#changed();
    void this.#sendDeletes(this.#epoch);
  }

  async #sendDeletes(epoch: number): Promise<void> {
    for (const id of this.#deletes.keys()) {
      try {
        // oxlint-disable-next-line no-await-in-loop -- deletes are rare; one at a time keeps errors simple
        await this.#a.remove(id);
      } catch (error) {
        if (epoch !== this.#epoch) return;
        if (!(error instanceof ApiError && error.status === 404)) {
          this.#deleteStatus = error instanceof ApiError && error.offline ? "offline" : "error";
          this.#deleteError = error instanceof Error ? error.message : String(error);
          this.#refreshStatus();
          return;
        }
      }
      if (epoch !== this.#epoch) return;
      this.#deletes.delete(id);
    }
    this.#deleteStatus = "saved";
    this.#deleteError = null;
    this.#changed();
  }

  #sync(init: {
    value: T;
    revision: number | null;
    unsaved: boolean;
    editsKnown: boolean;
  }): RecordSync<T> {
    const id = init.value.id;
    const epoch = this.#epoch;
    const sync: RecordSync<T> = new RecordSync<T>(
      {
        env: this.#env,
        debounceMs: this.#debounceMs,
        save: (value, revision) =>
          revision === null ? this.#a.create(value) : this.#a.update(value, revision),
        parse: (current) => parseStored(current, (raw) => this.#a.normalize(raw)),
        merge: (local, remote) => this.#a.merge(local, remote),
        onValue: (value) => {
          if (epoch === this.#epoch && this.#syncs.get(id) === sync) this.#setItem(id, value);
        },
        onStatus: () => {
          if (epoch === this.#epoch) this.#changed();
        },
        recreateWhenMissing: true,
      },
      init,
    );
    return sync;
  }

  #changed(): void {
    this.#persist();
    this.#refreshStatus();
  }

  #refreshStatus(): void {
    if (this.#expired) {
      this.#status.value = "error";
      this.#error.value = SESSION_ENDED;
      return;
    }
    if (this.#mode.value !== "account") {
      this.#status.value = "saved";
      this.#error.value = null;
      return;
    }
    const syncs = [...this.#syncs.values()];
    this.#status.value = worstStatus([this.#deleteStatus, ...syncs.map((s) => s.status)]);
    this.#error.value = syncs.find((s) => s.error !== null)?.error ?? this.#deleteError;
  }

  // ---- Drafts ----

  /** Writes the draft (signed out) or what the server doesn't have yet (signed in) to sessionStorage. */
  #persist(): void {
    if (this.#mode.value !== "account") {
      const items = Object.values(this.#items.value).map((value) => ({ value, basedOn: null }));
      writeJson(this.#env, this.#draftKey, items.length > 0 ? { items } : null);
      return;
    }
    const items: LocalCopy<T>[] = [];
    for (const sync of this.#syncs.values())
      if (sync.unsaved) items.push({ value: sync.value, basedOn: sync.revision });
    const deleted = [...this.#deletes].map(([id, revision]) => ({ id, revision }));
    const empty = items.length === 0 && deleted.length === 0;
    writeJson(this.#env, this.#pendingKey, empty ? null : { owner: this.#owner, items, deleted });
  }

  /** Records and deletions kept in storage under `key`; only `owner`'s when an owner is given. */
  #readLocal(key: string, owner: string | null): { items: LocalCopy<T>[]; deleted: string[] } {
    return readCollectionCopy(this.#env, key, owner, (raw) => this.#a.normalize(raw));
  }

  // ---- Lifecycle ----

  #reset(mode: StoreMode): void {
    this.#epoch += 1;
    for (const sync of this.#syncs.values()) sync.stop();
    this.#syncs.clear();
    this.#deletes.clear();
    this.#deleteStatus = "saved";
    this.#deleteError = null;
    this.#expired = false;
    this.#mode.value = mode;
  }

  /** Signed out: shows this tab's draft. */
  startDraft(): void {
    this.#reset("draft");
    this.#owner = null;
    const items: Record<string, T> = {};
    for (const { value } of this.#readLocal(this.#draftKey, null).items) items[value.id] = value;
    this.#items.value = items;
    this.#refreshStatus();
  }

  /**
   * Signed in as `owner`: loads the account's records, then saves this tab's draft and anything an earlier visit
   * didn't get to save. While loading, edits go to the draft and are saved with it. Rejects when the records can't
   * be loaded; the store then stays in "loading" with the draft on screen.
   */
  async startAccount(owner: string): Promise<void> {
    if (this.#mode.value === "account") this.#items.value = {};
    this.#reset("loading");
    this.#owner = owner;
    this.#refreshStatus();
    const epoch = this.#epoch;
    const remote = await this.#a.list();
    if (epoch !== this.#epoch) return;

    const pending = this.#readLocal(this.#pendingKey, owner);
    const draft = this.#readLocal(this.#draftKey, null);
    const items: Record<string, T> = {};
    const revisions = new Map<string, number>();
    this.#mode.value = "account";
    for (const r of remote) {
      items[r.value.id] = r.value;
      revisions.set(r.value.id, r.revision);
      this.#syncs.set(
        r.value.id,
        this.#sync({ value: r.value, revision: r.revision, unsaved: false, editsKnown: true }),
      );
    }
    // Earlier unsaved changes first, then the signed-out draft on top.
    for (const local of pending.items) this.#importLocal(items, revisions, local);
    for (const local of draft.items) this.#importLocal(items, revisions, local, true);
    for (const id of pending.deleted) {
      if (items[id] === undefined) continue;
      delete items[id];
      this.#syncs.get(id)?.stop();
      this.#syncs.delete(id);
      this.#deletes.set(id, revisions.get(id) ?? null);
    }
    this.#items.value = items;
    writeJson(this.#env, this.#draftKey, null);
    this.#changed();
    if (this.#deletes.size > 0) void this.#sendDeletes(epoch);
  }

  /**
   * Folds a record kept on this device into the account's: new ones are created, known ones merged and saved. A new
   * signed-out draft goes through the adapter's `adoptDraft` first.
   */
  #importLocal(
    items: Record<string, T>,
    revisions: Map<string, number>,
    local: LocalCopy<T>,
    isDraft = false,
  ): void {
    const id = local.value.id;
    const stored = items[id];
    const revision = revisions.get(id) ?? null;
    if (isDraft && stored === undefined && this.#a.adoptDraft !== undefined) {
      const adopted = this.#a.adoptDraft(local.value, Object.values(items));
      if (adopted === null) return;
      items[id] = adopted.value;
      this.#syncs.set(
        id,
        this.#sync({
          value: adopted.value,
          revision: null,
          unsaved: adopted.save,
          editsKnown: true,
        }),
      );
      return;
    }
    // Based on what the server still has: this device's copy is simply newer. Otherwise merge the two.
    const next =
      stored !== undefined && revision !== local.basedOn
        ? this.#a.merge(local.value, stored)
        : local.value;
    if (next === stored) return;
    items[id] = next;
    this.#syncs.get(id)?.stop();
    this.#syncs.set(id, this.#sync({ value: next, revision, unsaved: true, editsKnown: false }));
  }

  /** The session ended unexpectedly: stops writing, keeps everything on screen and the unsaved part in storage. */
  expire(): void {
    if (this.#mode.value !== "account") return;
    this.#persist();
    this.#epoch += 1;
    for (const sync of this.#syncs.values()) sync.stop();
    this.#expired = true;
    this.#refreshStatus();
  }

  /** Signed out on purpose: forgets the account's records and anything unsaved, and starts an empty draft. */
  clear(): void {
    this.#reset("draft");
    writeJson(this.#env, this.#pendingKey, null);
    writeJson(this.#env, this.#draftKey, null);
    this.#owner = null;
    this.#items.value = {};
    this.#refreshStatus();
  }

  /** Tries every failed or offline write again. */
  retry(): void {
    if (this.#mode.value !== "account" || this.#expired) return;
    for (const sync of this.#syncs.values()) sync.retry();
    if (this.#deletes.size > 0) void this.#sendDeletes(this.#epoch);
  }

  /** Writes everything now, without waiting out the pause. */
  async flush(): Promise<void> {
    if (this.#mode.value !== "account" || this.#expired) return;
    await Promise.all([...this.#syncs.values()].map((s) => s.flush()));
    if (this.#deletes.size > 0) await this.#sendDeletes(this.#epoch);
  }
}
