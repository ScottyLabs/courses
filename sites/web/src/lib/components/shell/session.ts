// What the app shell needs to know about the signed-in student. The data
// layer provides the real implementation; the root layout calls `setSession`
// once and the header, menus and sign-in prompts read it with `getSession`.
import { createContext } from "svelte";

export type SessionStatus = "loading" | "signed-out" | "signed-in";

/** Whether the student's changes have reached their account (the data layer's save status). */
export interface ShellSync {
  readonly status: "saved" | "saving" | "offline" | "error";
  /** What went wrong, when status is "error". */
  readonly error: string | null;
  /** Tries the failed or waiting writes again. */
  retry: () => void;
}

export interface ShellSession {
  /**
   * "loading" until the first session check finishes. Implementations should
   * back these fields with runes state (or getters over it) so the shell updates.
   */
  readonly status: SessionStatus;
  /** Display name while signed in, such as "Andrew Carnegie". */
  readonly name?: string;
  /** Second line under the name on the profile, such as a department. */
  readonly detail?: string;
  /** Starts sign-in (navigates to the identity provider). */
  signIn: () => void;
  /** Signs out and returns to the site signed out. */
  signOut: () => void;
  /** Save status while signed in, for the header's indicator; absent where nothing is saved. */
  readonly sync?: ShellSync;
}

export const [getSession, setSession] = createContext<ShellSession>();
