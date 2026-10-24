// Who's signed in. `GET /api/me` answers with the CMU user or 401; signing in is a full-page trip through
// `/auth/login` (Keycloak), and signing out is `POST /auth/logout`. Listeners hear about every change of account so
// catalog documents and personal data can reload for the new session.
import { ApiError } from "../api/errors.ts";
import type { Api } from "../api/endpoints.ts";
import type { Me } from "../api/types.ts";
import type { CellFactory, Cell } from "./cell.ts";

/**
 * loading: not known yet; signed-in; signed-out; unavailable: the API couldn't be reached, so account features
 * can't work right now (the catalog may still load).
 */
export type SessionStatus = "loading" | "signed-in" | "signed-out" | "unavailable";

/** Why the last sign-in didn't take: the account isn't a verified CMU one (the API answered 403). */
export type AuthError = "not-cmu";

/** The parts of the browser the session uses, so tests can stand in for them. */
export interface SessionEnvironment {
  /** The current path with its query and hash, to come back to after signing in. */
  currentPath(): string;
  /** Leaves the page for `url`. */
  navigate(url: string): void;
  /** sessionStorage, or null when it's unavailable. */
  storage(): Storage | null;
}

const RETURN_KEY = "cmucourses:return-to";

/** A path on this site, or "/" for anything else (no other origins, no `//host`, no auth routes). */
export function safeReturnPath(path: string | null | undefined): string {
  if (
    typeof path !== "string" ||
    !path.startsWith("/") ||
    path.startsWith("//") ||
    path.includes("\\")
  )
    return "/";
  if (/^\/auth(?:[/?#]|$)/u.test(path)) return "/";
  return path;
}

/** Where "Sign in" goes: the API runs Keycloak's sign-in and comes back to `returnTo`. */
export function signInUrl(returnTo: string): string {
  return `/auth/login?returnTo=${encodeURIComponent(safeReturnPath(returnTo))}`;
}

/** `initial` is true for the first answer after the page loads. */
type Listener = (user: Me | null, change: { initial: boolean }) => void;

export class Session {
  readonly #api: Api;
  readonly #env: SessionEnvironment;
  readonly #status: Cell<SessionStatus>;
  readonly #user: Cell<Me | null>;
  readonly #authError: Cell<AuthError | null>;
  readonly #listeners = new Set<Listener>();
  readonly #beforeSignIn = new Set<() => void>();
  #refreshing: Promise<void> | null = null;
  #answered = false;

  constructor(api: Api, cell: CellFactory, env: SessionEnvironment) {
    this.#api = api;
    this.#env = env;
    this.#status = cell<SessionStatus>("loading");
    this.#user = cell<Me | null>(null);
    this.#authError = cell<AuthError | null>(null);
  }

  get status(): SessionStatus {
    return this.#status.value;
  }

  get user(): Me | null {
    return this.#user.value;
  }

  get signedIn(): boolean {
    return this.#status.value === "signed-in";
  }

  get authError(): AuthError | null {
    return this.#authError.value;
  }

  /**
   * Hears about the first answer and every change of account after it: a user when signed in, null when signed out.
   * Returns an unsubscribe.
   */
  onChange(listener: Listener): () => void {
    this.#listeners.add(listener);
    return () => {
      this.#listeners.delete(listener);
    };
  }

  /** Runs right before the page leaves to sign in (the stores keep unsaved changes as drafts). */
  beforeSignIn(hook: () => void): () => void {
    this.#beforeSignIn.add(hook);
    return () => {
      this.#beforeSignIn.delete(hook);
    };
  }

  /** Asks the API who's signed in. Concurrent calls share one request. */
  refresh(): Promise<void> {
    this.#refreshing ??= this.#load().finally(() => {
      this.#refreshing = null;
    });
    return this.#refreshing;
  }

  async #load(): Promise<void> {
    try {
      const me = await this.#api.me();
      this.#authError.value = null;
      this.#set("signed-in", me);
    } catch (error) {
      if (error instanceof ApiError && error.status === 401) this.#set("signed-out", null);
      else if (error instanceof ApiError && error.status === 403) {
        this.#authError.value = "not-cmu";
        this.#set("signed-out", null);
      } else if (this.#status.value === "loading") this.#set("unavailable", null);
      // Otherwise (a hiccup mid-session) keep what we knew.
    }
  }

  #set(status: SessionStatus, user: Me | null): void {
    const before = this.#user.value?.subject ?? null;
    const initial = !this.#answered;
    this.#answered = true;
    this.#status.value = status;
    this.#user.value = user;
    const after = user?.subject ?? null;
    // The first answer counts as a change too, so listeners load the signed-in user's data.
    if (before !== after || initial)
      for (const listener of this.#listeners) listener(user, { initial });
  }

  /** The session ended on the server (a 401 from an API call while signed in). */
  expired(): void {
    if (this.#status.value === "signed-in") this.#set("signed-out", null);
  }

  /** Leaves for the sign-in page, coming back to `returnTo` (default: this page). */
  signIn(returnTo = this.#env.currentPath()): void {
    for (const hook of this.#beforeSignIn) hook();
    const path = safeReturnPath(returnTo);
    try {
      this.#env.storage()?.setItem(RETURN_KEY, path);
    } catch {
      // Storage off: the API's own returnTo still brings the student back.
    }
    this.#env.navigate(signInUrl(path));
  }

  /** Signs out on the server, then here. False when the server couldn't be reached (still signed in there). */
  async signOut(): Promise<boolean> {
    try {
      await this.#api.signOut();
    } catch (error) {
      if (!(error instanceof ApiError && error.status === 401)) return false;
    }
    this.#set("signed-out", null);
    return true;
  }

  /**
   * The page the student was on before signing in, once, when it differs from where they are now. For the app
   * shell to go back to after the sign-in round trip, in case the API didn't.
   */
  consumeReturnPath(): string | null {
    let path: string | null = null;
    try {
      const storage = this.#env.storage();
      path = storage?.getItem(RETURN_KEY) ?? null;
      storage?.removeItem(RETURN_KEY);
    } catch {
      return null;
    }
    if (path === null || !this.signedIn) return null;
    const safe = safeReturnPath(path);
    return safe === this.#env.currentPath() ? null : safe;
  }
}

/** The browser, for the website's session. */
export const browserEnvironment: SessionEnvironment = {
  currentPath: () =>
    typeof location === "undefined"
      ? "/"
      : `${location.pathname}${location.search}${location.hash}`,
  navigate: (url) => {
    location.assign(url);
  },
  storage: () => {
    try {
      return typeof sessionStorage === "undefined" ? null : sessionStorage;
    } catch {
      return null;
    }
  },
};
