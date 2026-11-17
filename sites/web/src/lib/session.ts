import { derived, writable } from "svelte/store";

type Session =
  | { status: "loading" | "signed-out" | "unavailable" }
  | { status: "signed-in"; name: string; canAccessMemberCatalog: boolean };

const state = writable<Session>({ status: "loading" });
export const session = { subscribe: state.subscribe };
export const loggedIn = derived(
  session,
  ($session) => $session.status === "signed-in",
);

let pending: Promise<void> | undefined;

export function refreshSession(): Promise<void> {
  pending ??= readSession().then(state.set).finally(() => {
    pending = undefined;
  });
  return pending;
}

async function readSession(): Promise<Session> {
  try {
    const response = await fetch("/auth/status", {
      credentials: "same-origin",
      cache: "no-store",
    });
    if (response.ok) {
      const payload: unknown = await response.json();
      if (
        typeof payload === "object" && payload !== null && "name" in payload
      ) {
        if (payload.name === null) return { status: "signed-out" };
        if (typeof payload.name === "string") {
          return {
            status: "signed-in",
            name: payload.name,
            canAccessMemberCatalog: "can_access_member_catalog" in payload &&
              payload.can_access_member_catalog === true,
          };
        }
      }
    }
  } catch {
    // The UI can retry if the session endpoint is unavailable.
  }
  return { status: "unavailable" };
}
