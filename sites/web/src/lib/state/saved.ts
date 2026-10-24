// Saved courses (`/api/saved`): bookmarks that aren't tied to a semester. Courseweave kept them per semester; here
// they're one list. Each change is its own small request, so there are no revisions: the last change to a course
// wins. Signed out they're a draft in sessionStorage, and unsent changes are kept there until the server has them.
import { ApiError } from "../api/errors.ts";
import type { Api } from "../api/endpoints.ts";
import type { Cell } from "../data/cell.ts";
import { SESSION_ENDED, type StoreDeps, type StoreMode } from "./drafts.ts";
import { isObject, readJson, writeJson, type StoreEnv } from "./env.ts";
import type { SaveStatus } from "./sync.ts";

const DRAFT_KEY = "cmucourses:draft:saved";
const PENDING_KEY = "cmucourses:pending:saved";

const strings = (v: unknown): string[] =>
  Array.isArray(v) ? v.filter((x): x is string => typeof x === "string") : [];

export class SavedStore {
  readonly #api: Api;
  readonly #env: StoreEnv;
  readonly #ids: Cell<string[]>;
  readonly #mode: Cell<StoreMode>;
  readonly #status: Cell<SaveStatus>;
  readonly #error: Cell<string | null>;
  /** Changes the server doesn't have yet: course id to saved (true) or not (false). */
  readonly #pending = new Map<string, boolean>();
  #sending: Promise<void> | null = null;
  #timer: unknown = null;
  #owner: string | null = null;
  #expired = false;
  #epoch = 0;

  constructor(api: Api, deps: StoreDeps) {
    this.#api = api;
    this.#env = deps.env;
    this.#ids = deps.cell<string[]>([]);
    this.#mode = deps.cell<StoreMode>("draft");
    this.#status = deps.cell<SaveStatus>("saved");
    this.#error = deps.cell<string | null>(null);
  }

  /** Saved course ids, oldest first. */
  get ids(): string[] {
    return this.#ids.value;
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

  get unsaved(): boolean {
    return this.#mode.value === "account" && (this.#pending.size > 0 || this.#sending !== null);
  }

  has(courseId: string): boolean {
    return this.#ids.value.includes(courseId);
  }

  /** Saves or unsaves a course. Returns whether it's saved now. */
  toggle(courseId: string): boolean {
    const saved = !this.has(courseId);
    this.set(courseId, saved);
    return saved;
  }

  add(courseId: string): void {
    this.set(courseId, true);
  }

  remove(courseId: string): void {
    this.set(courseId, false);
  }

  set(courseId: string, saved: boolean): void {
    if (this.has(courseId) === saved) return;
    this.#ids.value = saved
      ? [...this.#ids.value, courseId]
      : this.#ids.value.filter((id) => id !== courseId);
    if (this.#mode.value === "account" && !this.#expired) {
      this.#pending.set(courseId, saved);
      void this.#send();
    }
    this.#persist();
  }

  #persist(): void {
    if (this.#mode.value !== "account") {
      const ids = this.#ids.value;
      writeJson(this.#env, DRAFT_KEY, ids.length > 0 ? { ids } : null);
      return;
    }
    const ops = Object.fromEntries(this.#pending);
    writeJson(this.#env, PENDING_KEY, this.#pending.size > 0 ? { owner: this.#owner, ops } : null);
  }

  #setStatus(status: SaveStatus, error: string | null): void {
    this.#status.value = this.#expired ? "error" : status;
    this.#error.value = this.#expired ? SESSION_ENDED : error;
  }

  #send(): Promise<void> {
    this.#sending ??= this.#sendAll(this.#epoch).finally(() => {
      this.#sending = null;
    });
    return this.#sending;
  }

  async #sendAll(epoch: number): Promise<void> {
    while (this.#pending.size > 0 && epoch === this.#epoch && !this.#expired) {
      if (!this.#env.online()) {
        this.#setStatus("offline", null);
        return;
      }
      const [courseId, saved] = [...this.#pending][0] ?? ["", false];
      this.#setStatus("saving", null);
      try {
        // oxlint-disable-next-line no-await-in-loop -- one change at a time keeps them in order
        await (saved ? this.#api.saved.add(courseId) : this.#api.saved.remove(courseId));
      } catch (error) {
        if (epoch !== this.#epoch) return;
        if (!(error instanceof ApiError && error.status === 404 && !saved)) {
          this.#failed(error);
          return;
        }
      }
      if (epoch !== this.#epoch) return;
      // A newer change to the same course goes out next; this one is done.
      if (this.#pending.get(courseId) === saved) this.#pending.delete(courseId);
      this.#persist();
    }
    if (epoch === this.#epoch) this.#setStatus("saved", null);
  }

  #failed(error: unknown): void {
    if (error instanceof ApiError && error.offline) this.#setStatus("offline", null);
    else if (error instanceof ApiError && error.rateLimited) {
      const seconds = error.retryAfter ?? 60;
      this.#setStatus("error", "Saving paused for a moment. Your changes are still here.");
      if (this.#timer !== null) this.#env.clearTimeout(this.#timer);
      this.#timer = this.#env.setTimeout(() => {
        this.#timer = null;
        void this.#send();
      }, seconds * 1000);
    } else this.#setStatus("error", error instanceof Error ? error.message : String(error));
  }

  #reset(mode: StoreMode): void {
    this.#epoch += 1;
    this.#pending.clear();
    if (this.#timer !== null) this.#env.clearTimeout(this.#timer);
    this.#timer = null;
    this.#expired = false;
    this.#mode.value = mode;
    this.#setStatus("saved", null);
  }

  /** Signed out: shows this tab's draft. */
  startDraft(): void {
    this.#reset("draft");
    this.#owner = null;
    const raw = readJson(this.#env, DRAFT_KEY);
    this.#ids.value = isObject(raw) ? [...new Set(strings(raw.ids))] : [];
  }

  /** Signed in as `owner`: loads the account's saved courses, then sends this tab's draft and unsent changes. */
  async startAccount(owner: string): Promise<void> {
    if (this.#mode.value === "account") this.#ids.value = [];
    this.#reset("loading");
    this.#owner = owner;
    const epoch = this.#epoch;
    const remote = await this.#api.saved.list();
    if (epoch !== this.#epoch) return;
    const ids = remote.toSorted((a, b) => a.savedAt - b.savedAt).map((s) => s.courseId);
    const pending = readJson(this.#env, PENDING_KEY);
    if (isObject(pending) && pending.owner === owner && isObject(pending.ops)) {
      for (const [courseId, saved] of Object.entries(pending.ops)) {
        if (typeof saved === "boolean" && /^\d{5}$/u.test(courseId))
          this.#pending.set(courseId, saved);
      }
    }
    // Courses saved signed out (or while the account was loading) are saved to the account.
    const draft = readJson(this.#env, DRAFT_KEY);
    for (const courseId of [...(isObject(draft) ? strings(draft.ids) : []), ...this.#ids.value]) {
      this.#pending.set(courseId, true);
    }
    const next = ids.filter((id) => this.#pending.get(id) !== false);
    for (const [courseId, saved] of this.#pending) {
      if (saved && !next.includes(courseId)) next.push(courseId);
      else if (saved) this.#pending.delete(courseId);
    }
    this.#mode.value = "account";
    this.#ids.value = next;
    writeJson(this.#env, DRAFT_KEY, null);
    this.#persist();
    if (this.#pending.size > 0) void this.#send();
  }

  /** The session ended unexpectedly: stops sending and keeps unsent changes in storage. */
  expire(): void {
    if (this.#mode.value !== "account") return;
    this.#persist();
    this.#epoch += 1;
    this.#expired = true;
    this.#setStatus("error", SESSION_ENDED);
  }

  /** Signed out on purpose: forgets the account's saved courses and unsent changes. */
  clear(): void {
    this.#reset("draft");
    writeJson(this.#env, PENDING_KEY, null);
    writeJson(this.#env, DRAFT_KEY, null);
    this.#owner = null;
    this.#ids.value = [];
  }

  retry(): void {
    if (this.#mode.value === "account" && !this.#expired && this.#pending.size > 0)
      void this.#send();
  }

  async flush(): Promise<void> {
    if (this.#mode.value === "account" && !this.#expired && this.#pending.size > 0)
      await this.#send();
  }
}
