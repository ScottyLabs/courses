// The website's catalog and session, reactive through Svelte's $state. Importing this module in the browser starts
// asking who's signed in.
import { api } from "../api/index.ts";
import { Catalog } from "./catalog.ts";
import { runeCell } from "./cell.svelte.ts";
import { browserEnvironment, Session } from "./session.ts";

/** Catalog documents for the current session. */
export const catalog = new Catalog(api, runeCell);

/** Who's signed in. */
export const session = new Session(api, runeCell, browserEnvironment);

// A 401 from anything but /api/me while signed in means the session ended on the server.
api.client.onUnauthorized((path) => {
  if (path !== "/api/me") session.expired();
});

// Signing out (or a session ending) changes which version of each document the server sends. The first answer
// doesn't: documents requested before it were already sent for the browser's actual session.
session.onChange((_user, { initial }) => {
  if (!initial) catalog.invalidate();
});

const firstAnswer = typeof window === "undefined" ? Promise.resolve() : session.refresh();

// Member-only documents wait for the first answer, then aren't requested when the server would refuse them.
catalog.setMemberGate(async () => {
  await firstAnswer;
  if (session.authError === "not-cmu") return "forbidden";
  return session.status === "signed-out" ? "signin" : null;
});
