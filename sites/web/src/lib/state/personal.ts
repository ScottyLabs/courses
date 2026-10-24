// The student's own data, together: schedules, degree plans, the academic record, saved courses and preferences.
// It follows the session (decision 17): signed out, every store is a draft in this tab; on signing in each loads the
// account's copy, and the schedules, plans, record and saved courses drafted signed out wait for the student to add
// them to that account or discard them (heldDrafts), since the tab may be on a shared computer; signing out forgets
// the account's data; a session that ends on its own keeps what's on screen and in storage so signing in again
// saves it.
import type { Api } from "../api/endpoints.ts";
import type { Term } from "../core/data.ts";
import { onboardingDue } from "../core/state/onboarding.ts";
import type { CellFactory, Cell } from "../data/cell.ts";
import type { Session } from "../data/session.ts";
import type { StoreMode } from "./drafts.ts";
import type { DevicePrefs } from "./device.ts";
import type { StoreEnv } from "./env.ts";
import { linkActions, type LinkActions } from "./link-actions.ts";
import { PlanStore } from "./plans.ts";
import { PreferencesStore } from "./preferences.ts";
import { RecordStore } from "./record.ts";
import { SavedStore } from "./saved.ts";
import { ScheduleStore } from "./schedules.ts";
import { worstStatus, type SaveStatus } from "./sync.ts";

export interface PersonalDeps {
  api: Api;
  session: Session;
  device: DevicePrefs;
  cell: CellFactory;
  env: StoreEnv;
  /** A semester's course data (the catalog's term()), for picking sections when a plan adds to a schedule. */
  loadTerm(id: string): Promise<Term>;
  /** Quiet time before a write, in milliseconds. */
  debounceMs?: number;
}

/** What signing out did: done; unsaved (changes couldn't be saved first; ask, then pass force); failed. */
export type SignOutResult = "done" | "unsaved" | "failed";

/** What this tab drafted signed out and holds back from the account the student signed in to. */
export interface HeldDrafts {
  /** Schedules with courses in them (empty defaults aren't asked about). */
  schedules: number;
  plans: number;
  /** Whether there's anything for the academic record. */
  record: boolean;
  /** Saved courses. */
  saved: number;
}

interface Lifecycle {
  readonly mode: StoreMode;
  readonly status: SaveStatus;
  readonly error: string | null;
  readonly unsaved: boolean;
  startDraft(): void;
  startAccount(owner: string): Promise<void>;
  expire(): void;
  clear(): void;
  retry(): void;
  flush(): Promise<void>;
}

const LOAD_FAILED = "Couldn't load your saved data. Check your connection and retry.";

export class Personal {
  readonly preferences: PreferencesStore;
  readonly schedules: ScheduleStore;
  readonly plans: PlanStore;
  readonly record: RecordStore;
  readonly saved: SavedStore;
  /** Degree plan edits that may land in a synced schedule. */
  readonly links: LinkActions;
  readonly #deps: PersonalDeps;
  readonly #stores: Lifecycle[];
  readonly #loadError: Cell<string | null>;
  #signingOut = false;
  #owner: string | null = null;
  #attached: (() => void) | null = null;

  constructor(deps: PersonalDeps) {
    this.#deps = deps;
    const storeDeps = { cell: deps.cell, env: deps.env, debounceMs: deps.debounceMs };
    this.preferences = new PreferencesStore(deps.api, storeDeps);
    this.schedules = new ScheduleStore(deps.api, this.preferences, storeDeps);
    this.plans = new PlanStore(deps.api, this.preferences, storeDeps);
    this.record = new RecordStore(deps.api, storeDeps);
    this.saved = new SavedStore(deps.api, storeDeps);
    this.links = linkActions({
      schedules: this.schedules,
      plans: this.plans,
      loadTerm: (id) => deps.loadTerm(id),
    });
    this.#stores = [
      this.preferences.store,
      this.schedules.store,
      this.plans.store,
      this.record.store,
      this.saved,
    ];
    this.#loadError = deps.cell<string | null>(null);
  }

  /**
   * Starts following the session: shows this tab's drafts now, and the account's data once signed in. Safe to call
   * more than once. Returns a function that stops following.
   */
  attach(): () => void {
    if (this.#attached !== null) return this.#attached;
    for (const store of this.#stores) store.startDraft();
    const { session, env } = this.#deps;
    const offChange = session.onChange((user) => {
      void this.#onSession(user?.subject ?? null);
    });
    const offOnline = env.onOnline(() => {
      this.retry();
    });
    // The session may have answered before this was attached.
    if (session.status === "signed-in" && session.user !== null)
      void this.#onSession(session.user.subject);
    this.#attached = () => {
      offChange();
      offOnline();
      this.#attached = null;
    };
    return this.#attached;
  }

  async #onSession(owner: string | null): Promise<void> {
    if (owner !== null) {
      if (owner === this.#owner && this.mode !== "draft") return;
      this.#owner = owner;
      await this.#load(owner);
      return;
    }
    this.#owner = null;
    this.#loadError.value = null;
    for (const store of this.#stores) {
      if (this.#signingOut) store.clear();
      else if (store.mode === "account") store.expire();
      else store.startDraft();
    }
  }

  async #load(owner: string): Promise<void> {
    this.#loadError.value = null;
    // What was drafted signed out waits for the student's say before it joins this account.
    if (this.schedules.store.mode !== "account")
      this.schedules.store.holdDraft((s) => s.entries.length > 0);
    if (this.plans.store.mode !== "account") this.plans.store.holdDraft(() => true);
    if (this.record.store.mode !== "account") this.record.store.holdDraft();
    if (this.saved.mode !== "account") this.saved.holdDraft();
    const results = await Promise.allSettled(
      this.#stores.map((store) =>
        store.mode === "account" ? Promise.resolve() : store.startAccount(owner),
      ),
    );
    if (owner === this.#owner && results.some((r) => r.status === "rejected"))
      this.#loadError.value = LOAD_FAILED;
  }

  /** draft: signed out (or the API is unreachable); loading: the account's data is loading; account: all loaded. */
  get mode(): StoreMode {
    const modes = this.#stores.map((s) => s.mode);
    if (modes.every((m) => m === "draft")) return "draft";
    if (modes.every((m) => m === "account")) return "account";
    return "loading";
  }

  /** The most pressing save status across the stores, for one indicator. */
  get status(): SaveStatus {
    if (this.#loadError.value !== null) return "error";
    return worstStatus(this.#stores.map((s) => s.status));
  }

  /** What went wrong, when status is "error". */
  get error(): string | null {
    return this.#loadError.value ?? this.#stores.find((s) => s.error !== null)?.error ?? null;
  }

  /** Signed in and the server doesn't have everything yet. */
  get unsaved(): boolean {
    return this.#stores.some((s) => s.unsaved);
  }

  /**
   * Signed in, with what this tab drafted signed out still held back from the account: what there is, or null when
   * there's nothing to ask about.
   */
  get heldDrafts(): HeldDrafts | null {
    if (this.mode !== "account") return null;
    const held: HeldDrafts = {
      schedules: this.schedules.store.held.length,
      plans: this.plans.store.held.length,
      record: this.record.store.held !== null,
      saved: this.saved.held.length,
    };
    return held.schedules + held.plans + held.saved > 0 || held.record ? held : null;
  }

  /** Adds the held drafts to the account: schedules and plans under names it doesn't use, the record merged in. */
  acceptDrafts(): void {
    this.schedules.store.adoptHeld();
    this.plans.store.adoptHeld();
    this.record.store.adoptHeld();
    this.saved.adoptHeld();
  }

  /** Forgets the held drafts. */
  discardDrafts(): void {
    this.schedules.store.dropHeld();
    this.plans.store.dropHeld();
    this.record.store.dropHeld();
    this.saved.dropHeld();
  }

  /** Whether to ask for the academic record now (core/state/onboarding.ts). */
  get onboardingDue(): boolean {
    const { session, device } = this.#deps;
    const signedIn = session.status === "signed-in";
    return onboardingDue({
      session: session.status,
      // A record drafted signed out may be on its way in.
      loaded: this.mode === "account" && this.heldDrafts === null,
      recordEmpty: this.record.isEmpty,
      asked: signedIn ? this.preferences.onboarding.asked : device.onboardingAsked,
    });
  }

  /** The student added their record or skipped the prompt: remembered by the account, or by this device. */
  markOnboardingAsked(): void {
    if (this.#deps.session.status === "signed-in") this.preferences.markOnboardingAsked();
    else this.#deps.device.markOnboardingAsked();
  }

  /** Tries failed writes again, and loading the account's data if that failed. */
  retry(): void {
    if (this.#loadError.value !== null && this.#owner !== null) {
      void this.#load(this.#owner);
      return;
    }
    for (const store of this.#stores) store.retry();
  }

  /** Writes everything now and waits up to `timeoutMs`. True when the account has everything. */
  async flush(timeoutMs = 15_000): Promise<boolean> {
    if (!this.unsaved) return true;
    let timer: unknown = null;
    const timeout = new Promise<void>((resolve) => {
      timer = this.#deps.env.setTimeout(resolve, timeoutMs);
    });
    await Promise.race([Promise.all(this.#stores.map((s) => s.flush())), timeout]);
    this.#deps.env.clearTimeout(timer);
    return !this.unsaved;
  }

  /**
   * Signs out after making sure the account has every change (unless `force`), then forgets the account's data in
   * this tab. "unsaved": some changes couldn't be saved; ask before signing out with `force`.
   */
  async signOut(force = false): Promise<SignOutResult> {
    if (!force && !(await this.flush())) return "unsaved";
    this.#signingOut = true;
    try {
      return (await this.#deps.session.signOut()) ? "done" : "failed";
    } finally {
      this.#signingOut = false;
    }
  }

  /** Deletes the account's data on the server, then signs out. False when the server couldn't do it. */
  async deleteAccount(): Promise<boolean> {
    try {
      await this.#deps.api.deleteAccount();
    } catch {
      return false;
    }
    return (await this.signOut(true)) === "done";
  }
}
