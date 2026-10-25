// The app shell's view of the session (components/shell/session.ts), backed by the real session and the personal
// data stores: signing out first saves what the account doesn't have yet.
import type { ShellSession } from "../components/shell/session.ts";
import type { Session } from "../data/session.ts";
import type { Personal } from "./personal.ts";

const UNSAVED =
  "Some of your changes haven't reached your account yet. Sign out anyway and lose them?";

export function shellSession(
  session: Session,
  personal: Personal,
  confirmLoss: (message: string) => boolean = (message) => globalThis.confirm(message),
): ShellSession {
  return {
    get status() {
      // "unavailable" (the API can't be reached) shows as signed out, with sign-in still offered.
      if (session.status === "loading") return "loading";
      return session.status === "signed-in" ? "signed-in" : "signed-out";
    },
    get name() {
      return session.user?.name;
    },
    get detail() {
      return session.user?.andrewId;
    },
    sync: {
      get status() {
        return personal.status;
      },
      get error() {
        return personal.error;
      },
      retry: () => {
        personal.retry();
      },
    },
    signIn: () => {
      session.signIn();
    },
    signOut: () => {
      void personal.signOut().then(async (result) => {
        if (result === "unsaved" && confirmLoss(UNSAVED)) await personal.signOut(true);
      });
    },
  };
}
