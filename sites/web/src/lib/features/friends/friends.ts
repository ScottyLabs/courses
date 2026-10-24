// Friends (new in CMU Courses; docs/src/api/web-api.md, "Friends"): accepted friends, requests either way, the
// invite link made in this tab and what the student shares. Friendships live on the server only, so this store
// loads them, applies each change once the server has it, and says what went wrong in words the page can show.
// Plain TypeScript with its state in cells (decision 27), so the tests run it against a fake API.
import { ApiError } from "../../api/errors.ts";
import type { Api } from "../../api/endpoints.ts";
import type { Friend, Invite, Sharing } from "../../api/types.ts";
import type { Cell, CellFactory } from "../../data/cell.ts";

export type FriendsApi = Pick<
  Api["friends"],
  "list" | "request" | "accept" | "remove" | "invite" | "revokeInvite" | "sharing" | "setSharing"
>;

export type LoadStatus = "idle" | "loading" | "ready" | "error";

/** What to tell the student about a failed request; `fallback` when the server didn't explain. */
export function friendlyError(error: unknown, fallback: string): string {
  if (!(error instanceof ApiError)) return fallback;
  if (error.offline) return "You're offline. Check your connection and try again.";
  if (error.rateLimited) return "Too many changes at once. Wait a moment and try again.";
  if (error.signInRequired) return "Your session ended. Sign in again to continue.";
  if (error.status >= 400 && error.status < 500 && error.message !== "") return error.message;
  return fallback;
}

/** "jdoe@andrew.cmu.edu", " JDoe " -> "jdoe": what the request route expects. */
export function normalizeAndrewId(input: string): string {
  return input.trim().toLowerCase().split("@")[0] ?? "";
}

/** Whether `input` can be an Andrew ID (as the server checks it). */
export function isAndrewId(input: string): boolean {
  return /^[a-z0-9_-]{1,64}$/u.test(normalizeAndrewId(input));
}

/** Friends sorted by name, as the server sends them, within each status. */
function byName(a: Friend, b: Friend): number {
  return a.name.localeCompare(b.name) || a.andrewId.localeCompare(b.andrewId);
}

export class FriendsStore {
  readonly #api: FriendsApi;
  readonly #list: Cell<Friend[]>;
  readonly #status: Cell<LoadStatus>;
  readonly #error: Cell<string | null>;
  readonly #sharing: Cell<Sharing | null>;
  readonly #invite: Cell<Invite | null>;
  #loading: Promise<void> | null = null;

  constructor(api: FriendsApi, cell: CellFactory) {
    this.#api = api;
    this.#list = cell<Friend[]>([]);
    this.#status = cell<LoadStatus>("idle");
    this.#error = cell<string | null>(null);
    this.#sharing = cell<Sharing | null>(null);
    this.#invite = cell<Invite | null>(null);
  }

  get status(): LoadStatus {
    return this.#status.value;
  }

  /** Why loading failed. */
  get error(): string | null {
    return this.#error.value;
  }

  get all(): Friend[] {
    return this.#list.value;
  }

  get accepted(): Friend[] {
    return this.#list.value.filter((f) => f.status === "accepted");
  }

  /** Requests waiting for the student's answer. */
  get incoming(): Friend[] {
    return this.#list.value.filter((f) => f.status === "incoming");
  }

  /** Requests the student sent that haven't been accepted. */
  get outgoing(): Friend[] {
    return this.#list.value.filter((f) => f.status === "outgoing");
  }

  /** What the student shows accepted friends; null until loaded. */
  get sharing(): Sharing | null {
    return this.#sharing.value;
  }

  /** The invite link made in this tab, until it expires or is revoked. The server can't list older ones. */
  get invite(): Invite | null {
    const invite = this.#invite.value;
    return invite !== null && invite.expiresAt > Date.now() ? invite : null;
  }

  /** Loads friends and sharing; concurrent calls share one load. */
  load(): Promise<void> {
    this.#loading ??= this.#load().finally(() => {
      this.#loading = null;
    });
    return this.#loading;
  }

  async #load(): Promise<void> {
    if (this.#status.value !== "ready") this.#status.value = "loading";
    try {
      const [list, sharing] = await Promise.all([this.#api.list(), this.#api.sharing()]);
      this.#list.value = list;
      this.#sharing.value = sharing;
      this.#error.value = null;
      this.#status.value = "ready";
    } catch (error) {
      this.#error.value = friendlyError(error, "Friends didn't load. Try again in a moment.");
      this.#status.value = "error";
    }
  }

  /** Forgets everything, for a change of account. */
  reset(): void {
    this.#list.value = [];
    this.#sharing.value = null;
    this.#invite.value = null;
    this.#error.value = null;
    this.#status.value = "idle";
  }

  /** Sends a request by Andrew ID. Rejects with an ApiError the page explains with friendlyError(). */
  async request(andrewId: string): Promise<Friend> {
    const friend = await this.#api.request(normalizeAndrewId(andrewId));
    this.#upsert(friend);
    return friend;
  }

  /** Accepts an incoming request. */
  async accept(id: string): Promise<void> {
    await this.#api.accept(id);
    this.#list.value = this.#list.value.map((f) =>
      f.id === id ? { ...f, status: "accepted" as const } : f,
    );
    // What they share comes with the list.
    void this.load();
  }

  /** Declines, cancels or removes; access ends at once on both sides. */
  async remove(id: string): Promise<void> {
    try {
      await this.#api.remove(id);
    } catch (error) {
      // Already gone (the other person removed it first): the result is the same.
      if (!(error instanceof ApiError && error.status === 404)) throw error;
    }
    this.#list.value = this.#list.value.filter((f) => f.id !== id);
  }

  /** Adds a friendship made elsewhere, such as by redeeming an invite. */
  added(friend: Friend): void {
    this.#upsert(friend);
  }

  /** Makes a new invite link, replacing the previous one. */
  async createInvite(): Promise<Invite> {
    const invite = await this.#api.invite();
    this.#invite.value = invite;
    return invite;
  }

  /** Revokes the invite made in this tab. */
  async revokeInvite(): Promise<void> {
    const invite = this.#invite.value;
    if (invite === null) return;
    await this.#api.revokeInvite(invite.token);
    this.#invite.value = null;
  }

  /** Changes what friends may see. Shows the change at once and puts it back if the server refuses. */
  async setSharing(patch: Partial<Sharing>): Promise<void> {
    const before = this.#sharing.value ?? { shareCourses: false, shareSchedules: false };
    const next = { ...before, ...patch };
    this.#sharing.value = next;
    try {
      this.#sharing.value = await this.#api.setSharing(next);
    } catch (error) {
      this.#sharing.value = before;
      throw error;
    }
  }

  #upsert(friend: Friend): void {
    const others = this.#list.value.filter((f) => f.id !== friend.id);
    this.#list.value = [...others, friend].toSorted(byName);
  }
}
