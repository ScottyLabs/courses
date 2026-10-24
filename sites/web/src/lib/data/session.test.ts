/* oxlint-disable typescript/no-floating-promises, eslint/max-lines-per-function -- node:test awaits describe and it itself; a describe groups many cases */
import { describe, it } from "node:test";
import { createMockServer } from "../../../dev/mock-server.ts";
import { ApiClient, type FetchLike } from "../api/client.ts";
import { createApi } from "../api/endpoints.ts";
import { MemoryStorage } from "../state/test-env.ts";
import { expect } from "../testing.ts";
import { isStaleBuildError } from "./app-update.ts";
import { plainCell } from "./cell.ts";
import { safeReturnPath, Session, signInUrl } from "./session.ts";

const forbidden: FetchLike = () =>
  Promise.resolve(
    Response.json({ code: "forbidden", message: "CMU accounts only" }, { status: 403 }),
  );

function setup(fetch: FetchLike, path = "/plan") {
  const storage = new MemoryStorage();
  const navigations: string[] = [];
  const current = { path };
  const session = new Session(createApi(new ApiClient({ fetch })), plainCell, {
    currentPath: () => current.path,
    navigate: (url) => {
      navigations.push(url);
    },
    storage: () => storage,
  });
  return { session, storage, navigations, current };
}

describe("Session", () => {
  it("knows who's signed in, and tells listeners once", async () => {
    const server = createMockServer({ catalogDir: null, signedIn: true });
    const { session } = setup(server.fetch);
    const heard: (string | null)[] = [];
    session.onChange((user, { initial }) => {
      heard.push(`${user?.andrewId ?? "nobody"}${initial ? " (first)" : ""}`);
    });
    expect(session.status).toBe("loading");
    await Promise.all([session.refresh(), session.refresh()]);
    expect(server.log.filter((l) => l === "GET /api/me")).toHaveLength(1);
    expect(session.status).toBe("signed-in");
    expect(session.user?.andrewId).toBe("scotty");
    await session.refresh();
    expect(heard).toEqual(["scotty (first)"]);
  });

  it("is signed out on 401, and says why on 403", async () => {
    const out = setup(createMockServer({ catalogDir: null }).fetch);
    await out.session.refresh();
    expect(out.session.status).toBe("signed-out");
    expect(out.session.authError).toBeNull();
    const other = setup(forbidden);
    await other.session.refresh();
    expect(other.session.status).toBe("signed-out");
    expect(other.session.authError).toBe("not-cmu");
  });

  it("is unavailable when the API can't be reached, until it can", async () => {
    let up = false;
    const server = createMockServer({ catalogDir: null, signedIn: true });
    const { session } = setup((url, init) =>
      up ? server.fetch(url, init) : Promise.reject(new TypeError("offline")),
    );
    await session.refresh();
    expect(session.status).toBe("unavailable");
    up = true;
    await session.refresh();
    expect(session.status).toBe("signed-in");
  });

  it("signs in through /auth/login and comes back to the page", async () => {
    const hooks: string[] = [];
    const { session, navigations, storage, current } = setup(
      createMockServer({ catalogDir: null }).fetch,
      "/plan?x=1",
    );
    session.beforeSignIn(() => {
      hooks.push("drafts kept");
    });
    session.signIn();
    expect(hooks).toEqual(["drafts kept"]);
    expect(navigations).toEqual([signInUrl("/plan?x=1")]);
    expect(navigations[0]).toBe("/auth/login?returnTo=%2Fplan%3Fx%3D1");
    expect(storage.getItem("cmucourses:return-to")).toBe("/plan?x=1");
    // Back from Keycloak, but on the home page.
    current.path = "/";
    expect(session.consumeReturnPath()).toBeNull();
    storage.setItem("cmucourses:return-to", "/plan?x=1");
    const server = createMockServer({ catalogDir: null, signedIn: true });
    const back = setup(server.fetch, "/");
    back.storage.setItem("cmucourses:return-to", "/plan?x=1");
    await back.session.refresh();
    expect(back.session.consumeReturnPath()).toBe("/plan?x=1");
    expect(back.session.consumeReturnPath()).toBeNull();
  });

  it("signs out on the server, and notices a session that ended", async () => {
    const server = createMockServer({ catalogDir: null, signedIn: true });
    const { session } = setup(server.fetch);
    await session.refresh();
    expect(await session.signOut()).toBe(true);
    expect(server.signedIn).toBe(false);
    expect(session.status).toBe("signed-out");
    server.signedIn = true;
    await session.refresh();
    session.expired();
    expect(session.status).toBe("signed-out");
  });

  it("only returns to paths on this site", () => {
    expect(safeReturnPath("/schedules?term=S27")).toBe("/schedules?term=S27");
    expect(safeReturnPath("https://evil.example/")).toBe("/");
    expect(safeReturnPath("//evil.example/")).toBe("/");
    expect(safeReturnPath("/auth/logout")).toBe("/");
    expect(safeReturnPath(null)).toBe("/");
  });
});

describe("isStaleBuildError", () => {
  it("recognizes a chunk from an older build", () => {
    expect(
      isStaleBuildError(
        new TypeError("Failed to fetch dynamically imported module: /_app/immutable/x.js"),
      ),
    ).toBe(true);
    expect(isStaleBuildError(new Error("Importing a module script failed."))).toBe(true);
    expect(isStaleBuildError(new Error("Unable to preload CSS for /_app/x.css"))).toBe(true);
    expect(isStaleBuildError(new Error("Cannot read properties of undefined"))).toBe(false);
  });
});
