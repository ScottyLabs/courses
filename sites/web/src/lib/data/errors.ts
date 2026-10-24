// Results of loading a member-only catalog document without a member's session, and a way to wait for any load
// without throwing.

/**
 * A member-only document asked for without a CMU session: "signin" when signed out (the API answered 401),
 * "forbidden" when signed in with an account that isn't a CMU one (403).
 */
export class SignInRequiredError extends Error {
  override readonly name = "SignInRequiredError";
  readonly document: string;
  readonly reason: "signin" | "forbidden";

  constructor(document: string, reason: "signin" | "forbidden" = "signin") {
    super(
      reason === "signin"
        ? "Sign in with your CMU account to see this."
        : "This is only for CMU accounts. Sign in with your Andrew ID to see it.",
    );
    this.document = document;
    this.reason = reason;
  }
}

/** A settled load: the value, sign in to see this (or a CMU account is needed), or what went wrong. */
export type Loaded<T> =
  | { status: "ready"; value: T }
  | { status: "signin" | "forbidden" }
  | { status: "error"; error: string };

/** Waits for a load and says how it went, without throwing. */
export async function settle<T>(promise: Promise<T>): Promise<Loaded<T>> {
  try {
    return { status: "ready", value: await promise };
  } catch (error) {
    if (error instanceof SignInRequiredError) return { status: error.reason };
    return { status: "error", error: error instanceof Error ? error.message : String(error) };
  }
}
