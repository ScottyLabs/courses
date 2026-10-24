/* oxlint-disable typescript/no-floating-promises, eslint/max-lines-per-function, typescript/no-unsafe-type-assertion -- node:test awaits describe and it itself; a describe groups many cases; responses are cast to their shapes */
import { describe, it } from "node:test";
import { expect } from "../src/lib/testing.ts";
import { catalogFile, createMockServer, publicTerm } from "./mock-server.ts";

const call = async (
  server: ReturnType<typeof createMockServer>,
  method: string,
  path: string,
  body?: unknown,
): Promise<{ status: number; json: unknown }> => {
  const init: RequestInit = { method };
  if (body !== undefined) {
    init.body = JSON.stringify(body);
    init.headers = { "Content-Type": "application/json" };
  }
  const response = await server.fetch(path, init);
  const text = await response.text();
  return { status: response.status, json: text === "" ? null : (JSON.parse(text) as unknown) };
};

describe("mock API", () => {
  it("maps document names to Courseweave's files", () => {
    expect(catalogFile("term/S27")).toBe("S27.json");
    expect(catalogFile("catalog/details/15")).toBe("catalog/details/15.json");
    expect(catalogFile("catalog/index")).toBe("catalog/index.json");
    expect(catalogFile("fce/courses/15")).toBe("fce/courses/15.json");
    expect(catalogFile("fce/summary")).toBe("fce/summary.json");
    expect(catalogFile("programs/19488")).toBe("programs/19488.json");
    expect(catalogFile("teaching")).toBe("teaching.json");
    expect(catalogFile("finals/S27")).toBe("finals/S27.json");
    expect(catalogFile("../secrets")).toBeNull();
    expect(catalogFile("term/S27/extra")).toBeNull();
  });

  it("leaves instructors and rooms out of public term documents", () => {
    const doc = {
      term: { id: "S27" },
      courses: [
        {
          id: "15122",
          instructors: ["A"],
          sections: [
            { instructors: ["A"], meetings: [{ building: "GHC", room: "4401", days: [1] }] },
          ],
        },
      ],
    };
    expect(publicTerm(doc)).toEqual({
      term: { id: "S27" },
      courses: [
        {
          id: "15122",
          instructors: [],
          sections: [{ instructors: [], meetings: [{ building: null, room: null, days: [1] }] }],
        },
      ],
    });
    expect(doc.courses[0]?.instructors).toEqual(["A"]);
  });

  it("keeps member documents and personal records for signed-in users", async () => {
    const server = createMockServer({ catalogDir: null });
    expect((await call(server, "GET", "/api/catalog/fce/summary")).status).toBe(401);
    expect((await call(server, "GET", "/api/schedules")).status).toBe(401);
    expect((await call(server, "GET", "/api/catalog/terms")).status).toBe(404);
  });

  it("revisions schedules and answers stale writes with 409 and the stored record", async () => {
    const server = createMockServer({ catalogDir: null, signedIn: true });
    const created = await call(server, "POST", "/api/schedules", {
      id: "s1",
      termId: "S27",
      name: "Mine",
    });
    expect(created.status).toBe(201);
    expect(created.json).toMatchObject({
      id: "s1",
      revision: 1,
      visibility: "private",
      entries: [],
      shareId: null,
    });
    expect(
      (await call(server, "POST", "/api/schedules", { id: "s1", termId: "S27", name: "Again" }))
        .status,
    ).toBe(409);
    expect(
      (await call(server, "POST", "/api/schedules", { termId: "Spring", name: "Bad" })).status,
    ).toBe(400);
    const update = {
      termId: "S27",
      name: "Renamed",
      entries: [],
      visibility: "friends",
      revision: 1,
    };
    expect((await call(server, "PUT", "/api/schedules/s1", update)).json).toMatchObject({
      name: "Renamed",
      revision: 2,
    });
    const stale = await call(server, "PUT", "/api/schedules/s1", update);
    expect(stale.status).toBe(409);
    expect(stale.json).toMatchObject({ code: "conflict", current: { revision: 2 } });
    const share = await call(server, "POST", "/api/schedules/s1/share");
    const { shareId } = share.json as { shareId: string };
    expect((await call(server, "GET", `/api/shared-schedules/${shareId}`)).json).toMatchObject({
      name: "Renamed",
    });
  });

  it("creates the record and preferences at revision 0", async () => {
    const server = createMockServer({ catalogDir: null, signedIn: true });
    expect((await call(server, "GET", "/api/record")).json).toMatchObject({
      entries: [],
      revision: 0,
    });
    const saved = await call(server, "PUT", "/api/record", {
      entries: [],
      overrides: {},
      profile: { college: "SCS" },
      revision: 0,
    });
    expect(saved.json).toMatchObject({ revision: 1, profile: { college: "SCS" } });
    expect(
      (
        await call(server, "PUT", "/api/record", {
          entries: [],
          overrides: {},
          profile: {},
          revision: 0,
        })
      ).status,
    ).toBe(409);
    expect((await call(server, "GET", "/api/preferences")).json).toMatchObject({
      planLayout: "cards",
      revision: 0,
    });
  });

  it("saves courses without revisions", async () => {
    const server = createMockServer({ catalogDir: null, signedIn: true, now: () => 5 });
    expect((await call(server, "PUT", "/api/saved/15122")).status).toBe(204);
    expect((await call(server, "PUT", "/api/saved/15122")).status).toBe(204);
    expect((await call(server, "GET", "/api/saved")).json).toEqual([
      { courseId: "15122", savedAt: 5 },
    ]);
    expect((await call(server, "DELETE", "/api/saved/15122")).status).toBe(204);
    expect((await call(server, "PUT", "/api/saved/abc")).status).toBe(400);
  });

  it("signs in and out like the API's development sign-in", async () => {
    const server = createMockServer({ catalogDir: null });
    const login = await server.fetch("/auth/login?as=bob&returnTo=%2Fplan", {});
    expect(login.status).toBe(303);
    expect(login.headers.get("Location")).toBe("/plan");
    expect((await call(server, "GET", "/api/me")).json).toMatchObject({ andrewId: "bob" });
    expect((await call(server, "POST", "/auth/logout")).status).toBe(204);
    expect((await call(server, "GET", "/api/me")).status).toBe(401);
    const evil = await server.fetch("/auth/login?returnTo=%2F%2Fevil.example", {});
    expect(evil.headers.get("Location")).toBe("/");
  });
});
