// Saving one server record (a schedule, a plan, the academic record, preferences) per decision 17. Edits apply here
// at once and are written back after a short pause, naming the revision they're based on. When another device wrote
// first (409), the edits made since that revision are replayed onto the stored record and the result is written
// again; when they aren't known (a draft carried over from before a reload), Courseweave's merge rules
// (core/sync-merge.ts) combine the two copies instead. Network trouble, rate limits and server errors back off and
// retry; nothing is dropped.
import { ApiError } from "../api/errors.ts";
import { retryIn, worstStatus, type Edit, type SaveStatus, type Stored } from "./drafts.ts";
import type { StoreEnv } from "./env.ts";

export { worstStatus, type Edit, type SaveStatus, type Stored };

export interface RecordSyncOptions<T> {
  env: StoreEnv;
  /** Quiet time before a write, so bursts of edits go out as one. */
  debounceMs: number;
  /** Writes `value`: creates it when `revision` is null, else replaces revision `revision`. */
  save(value: T, revision: number | null): Promise<Stored<T>>;
  /** The stored record from a 409's `current`, or null when it isn't one. */
  parse(current: unknown): Stored<T> | null;
  /** This device's value combined with the stored one, when the edits that made it aren't known. */
  merge(local: T, remote: T): T;
  /** Hears every change to the value that didn't come from edit(): a merge, or the server's copy after a save. */
  onValue(value: T): void;
  /** Hears every status change. */
  onStatus(): void;
  /** On 404: false gives up with an error; true writes the record again as new (it was deleted elsewhere). */
  recreateWhenMissing: boolean;
}

export interface RecordSyncInit<T> {
  value: T;
  /** The stored revision the value is based on; null when it has never been saved. */
  revision: number | null;
  /** The value has changes the server doesn't have. */
  unsaved: boolean;
  /** False when the value came from a draft whose edits aren't known: conflicts then merge instead of replaying. */
  editsKnown: boolean;
}

/** Retries after a server error start at this wait and double up to the cap, in milliseconds. */
const FIRST_BACKOFF_MS = 30_000;
const MAX_BACKOFF_MS = 10 * 60_000;
/** Conflicts in a row before giving up with an error (another device keeps writing). */
const MAX_CONFLICTS = 3;

const replay = <T>(edits: Edit<T>[], value: T): T => edits.reduce((v, edit) => edit(v), value);

export class RecordSync<T> {
  readonly #o: RecordSyncOptions<T>;
  #value: T;
  #revision: number | null;
  /** Edits since the value the server has (or null: unknown), replayed onto the stored record on a conflict. */
  #edits: Edit<T>[] | null;
  #dirty: boolean;
  #status: SaveStatus = "saved";
  #error: string | null = null;
  #timer: unknown = null;
  #running: Promise<void> | null = null;
  #stopped = false;
  #backoff = FIRST_BACKOFF_MS;
  #conflicts = 0;
  #pausedUntil = 0;
  #sent = false;

  constructor(options: RecordSyncOptions<T>, init: RecordSyncInit<T>) {
    this.#o = options;
    this.#value = init.value;
    this.#revision = init.revision;
    this.#edits = init.editsKnown ? [] : null;
    this.#dirty = init.unsaved;
    if (init.unsaved) this.#schedule();
  }

  get value(): T {
    return this.#value;
  }

  get revision(): number | null {
    return this.#revision;
  }

  get status(): SaveStatus {
    return this.#status;
  }

  /** A message for status "error". */
  get error(): string | null {
    return this.#error;
  }

  /** A write was ever sent, so the server may have the record even if no answer came back. */
  get sent(): boolean {
    return this.#sent || this.#revision !== null;
  }

  /** The server doesn't have everything yet (a write is waiting, running or failed). */
  get unsaved(): boolean {
    return this.#dirty || this.#running !== null;
  }

  /** Applies an edit now and saves it after the pause. Returns the new value. */
  edit(fn: Edit<T>): T {
    const next = fn(this.#value);
    if (next === this.#value) return next;
    this.#value = next;
    this.#edits?.push(fn);
    this.#dirty = true;
    this.#schedule();
    return next;
  }

  /**
   * Changes the value without saving it: for fields the server sets itself (a schedule's share link). Not replayed
   * on a conflict, since the server's copy already has them.
   */
  amend(fn: Edit<T>): T {
    this.#value = fn(this.#value);
    return this.#value;
  }

  /** Saves without waiting out the pause (or a back-off). */
  flush(): Promise<void> {
    this.#clearTimer();
    this.#pausedUntil = 0;
    return this.#start();
  }

  /** Tries again after an error or going offline. */
  retry(): void {
    if (this.#dirty) void this.flush();
  }

  /** Stops writing; a write under way finishes but its answer is ignored. */
  stop(): void {
    this.#stopped = true;
    this.#clearTimer();
  }

  /** Resolves once no write is running. */
  async settled(): Promise<void> {
    // oxlint-disable-next-line no-await-in-loop -- a write may start another (edits made meanwhile)
    while (this.#running !== null) await this.#running;
  }

  #clearTimer(): void {
    if (this.#timer !== null) this.#o.env.clearTimeout(this.#timer);
    this.#timer = null;
  }

  #schedule(delay = this.#o.debounceMs): void {
    if (this.#stopped) return;
    this.#clearTimer();
    const wait = Math.max(delay, this.#pausedUntil - this.#o.env.now());
    this.#timer = this.#o.env.setTimeout(() => {
      this.#timer = null;
      void this.#start();
    }, wait);
    // While a write is merely waiting, show "saving" (unless a problem is already being shown).
    if (this.#status === "saved") this.#setStatus("saving", null);
  }

  #setStatus(status: SaveStatus, error: string | null): void {
    if (this.#status === status && this.#error === error) return;
    this.#status = status;
    this.#error = error;
    this.#o.onStatus();
  }

  #start(): Promise<void> {
    if (this.#running !== null) return this.#running;
    if (!this.#dirty || this.#stopped) return Promise.resolve();
    this.#running = this.#run().finally(() => {
      this.#running = null;
      if (this.#stopped) return;
      if (!this.#dirty) this.#setStatus("saved", null);
      this.#o.onStatus();
    });
    return this.#running;
  }

  async #run(): Promise<void> {
    while (this.#dirty && !this.#stopped) {
      if (!this.#o.env.online()) {
        this.#setStatus("offline", null);
        return;
      }
      const snapshot = this.#value;
      const sent = this.#edits?.length ?? 0;
      this.#dirty = false;
      this.#sent = true;
      this.#setStatus("saving", null);
      try {
        // oxlint-disable-next-line no-await-in-loop -- each write depends on the answer to the one before
        const stored = await this.#o.save(snapshot, this.#revision);
        // Kept even when stopped, so a record deleted mid-create can still be deleted on the server.
        this.#revision = stored.revision;
        if (this.#stopped) return;
        this.#saved(stored, snapshot, sent);
      } catch (error) {
        if (this.#stopped) return;
        this.#dirty = true;
        if (!this.#failed(error)) return;
      }
    }
  }

  #saved(stored: Stored<T>, snapshot: T, sent: number): void {
    this.#backoff = FIRST_BACKOFF_MS;
    this.#conflicts = 0;
    if (this.#value === snapshot) {
      // Nothing changed meanwhile: adopt the server's copy (it sets timestamps).
      this.#edits = [];
      this.#value = stored.value;
      this.#o.onValue(stored.value);
    } else if (this.#edits !== null) {
      // Edits made while the write was out: replay them onto what the server now has.
      this.#edits = this.#edits.slice(sent);
      this.#value = replay(this.#edits, stored.value);
      this.#o.onValue(this.#value);
    }
  }

  /** Handles a failed write. True to try again at once (after a conflict was folded in). */
  #failed(error: unknown): boolean {
    if (!(error instanceof ApiError)) {
      this.#setStatus("error", error instanceof Error ? error.message : String(error));
      return false;
    }
    if (error.conflict) return this.#conflict(error);
    if (error.status === 404 && this.#o.recreateWhenMissing && this.#revision !== null) {
      // Deleted on another device while edited here: nothing on this device is dropped, so write it back.
      this.#revision = null;
      return true;
    }
    if (error.offline) {
      this.#setStatus("offline", null);
      return false;
    }
    if (error.signInRequired) {
      this.#setStatus("error", "Your session ended. Sign in again to save your changes.");
      return false;
    }
    if (error.rateLimited) {
      const seconds = error.retryAfter ?? 60;
      this.#pausedUntil = this.#o.env.now() + seconds * 1000;
      this.#setStatus(
        "error",
        `Saving paused, retrying ${retryIn(seconds)}. Your changes are still here.`,
      );
      this.#schedule(seconds * 1000);
      return false;
    }
    if (
      error.status === 400 ||
      error.status === 403 ||
      error.status === 404 ||
      error.status === 413
    ) {
      this.#setStatus("error", error.message);
      return false;
    }
    // A server error: retry later with growing gaps, never in a tight loop.
    this.#setStatus(
      "error",
      "Couldn't save to the server. Retrying shortly; your changes are still here.",
    );
    const wait = this.#backoff;
    this.#backoff = Math.min(this.#backoff * 2, MAX_BACKOFF_MS);
    this.#pausedUntil = this.#o.env.now() + wait;
    this.#schedule(wait);
    return false;
  }

  #conflict(error: ApiError): boolean {
    const remote = this.#o.parse(error.current);
    this.#conflicts += 1;
    if (remote === null || this.#conflicts > MAX_CONFLICTS) {
      this.#setStatus("error", "This keeps changing on another device. Try again in a moment.");
      return false;
    }
    this.#revision = remote.revision;
    this.#value =
      this.#edits === null
        ? this.#o.merge(this.#value, remote.value)
        : replay(this.#edits, remote.value);
    this.#o.onValue(this.#value);
    return true;
  }
}
