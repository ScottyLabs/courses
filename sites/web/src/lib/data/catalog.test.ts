/* oxlint-disable typescript/no-floating-promises, eslint/max-lines-per-function, typescript/no-unsafe-type-assertion, eslint/no-await-in-loop -- node:test awaits describe and it itself; a describe groups many cases; fixtures are cast to their shapes */
import { mkdir, mkdtemp, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { createMockServer, type MockServer } from "../../../dev/mock-server.ts";
import { ApiClient } from "../api/client.ts";
import { ApiError } from "../api/errors.ts";
import { createApi } from "../api/endpoints.ts";
import { fceMiniOf } from "../core/fce-minis.ts";
import { expect } from "../testing.ts";
import { Catalog, SignInRequiredError, settle } from "./catalog.ts";
import { plainCell } from "./cell.ts";

const meeting = { days: [1, 3], start: 600, end: 680, building: "GHC", room: "4401" };
const course = (id: string, title: string) => ({
  id,
  code: `${id.slice(0, 2)}-${id.slice(2)}`,
  title,
  units: 12,
  dept: "CS",
  level: "U",
  instructors: ["Ada Lovelace"],
  sections: [
    {
      name: "1",
      kind: "lecture",
      parent: null,
      mini: null,
      mode: "IPE",
      instructors: ["Ada Lovelace"],
      meetings: [meeting],
    },
  ],
  crossListed: [],
});

/** A small, synthetic catalog in Courseweave's file layout. */
async function catalogDir(): Promise<string> {
  const dir = await mkdtemp(join(tmpdir(), "cmucourses-catalog-"));
  const files: Record<string, unknown> = {
    "terms.json": [{ id: "S27", name: "Spring 2027", courseCount: 2, sectionCount: 2 }],
    "S27.json": {
      term: {
        id: "S27",
        name: "Spring 2027",
        season: "spring",
        year: 2027,
        minis: [],
        classesBegin: "2027-01-19",
        classesEnd: "2027-04-30",
        noClasses: [],
      },
      courses: [
        course("15122", "Principles of Imperative Computation"),
        course("15150", "Functional Programming"),
      ],
    },
    "catalog/index.json": { courses: {} },
    "catalog/details/15.json": { "15122": { description: "Imperative programming." } },
    "offerings.json": {
      terms: ["S27"],
      courses: {
        "15122": {
          title: "Principles of Imperative Computation",
          dept: "CS",
          level: "U",
          terms: ["S27"],
        },
      },
    },
    "fce/summary.json": {
      latest: "F26",
      recentTerms: 4,
      courses: { "15122": [10.5, 4.1, 4.2, 300, 4, 20, "F26", null, 400, 1, 1] },
    },
    "fce/courses/15.json": {
      "15122": [["F26", "1", "LOVELACE, ADA", 400, 300, 10.5, 4.2, 4.1, [4, 4, 4, 4, 4, 4, 4]]],
    },
    "programs/index.json": { programs: [] },
  };
  for (const [path, value] of Object.entries(files)) {
    await mkdir(join(dir, path, ".."), { recursive: true });
    await writeFile(join(dir, path), JSON.stringify(value));
  }
  return dir;
}

async function setup(signedIn: boolean): Promise<{ server: MockServer; catalog: Catalog }> {
  const server = createMockServer({ catalogDir: await catalogDir(), signedIn });
  return {
    server,
    catalog: new Catalog(createApi(new ApiClient({ fetch: server.fetch })), plainCell),
  };
}

describe("catalog documents", () => {
  it("requests each document once, however many ask at once", async () => {
    const { server, catalog } = await setup(false);
    const [a, b] = await Promise.all([catalog.term("S27"), catalog.term("S27")]);
    expect(a).toBe(b);
    await catalog.term("S27");
    expect(server.log.filter((l) => l === "GET /api/catalog/term/S27")).toHaveLength(1);
    expect(a.courseById.get("15150")?.title).toBe("Functional Programming");
  });

  it("gets the public version signed out: no instructors or rooms", async () => {
    const { catalog } = await setup(false);
    const term = await catalog.term("S27");
    const c = term.courseById.get("15122");
    expect(c?.instructors).toEqual([]);
    expect(c?.sections[0]?.meetings[0]?.room).toBeNull();
    expect(term.instructors).toEqual([]);
  });

  it("says sign in is required for member documents, and remembers it until invalidated", async () => {
    const { server, catalog } = await setup(false);
    await expect(catalog.fceSummary()).rejects.toBeInstanceOf(SignInRequiredError);
    expect(await catalog.fceAccess()).toBe("locked");
    expect(await settle(catalog.programsIndex())).toEqual({ status: "signin" });
    expect(server.log.filter((l) => l === "GET /api/catalog/fce/summary")).toHaveLength(1);

    server.signedIn = true;
    const before = catalog.generation;
    catalog.invalidate();
    expect(catalog.generation).toBe(before + 1);
    const summary = await catalog.fceSummary();
    expect(summary?.courses.get("15122")?.hours).toBe(10.5);
    expect(await catalog.fceAccess()).toBe("available");
    // The summary registers minis for pure code.
    expect(fceMiniOf("15122")).toEqual({ share: 1, half: "first" });
    const term = await catalog.term("S27");
    expect(term.courseById.get("15122")?.instructors).toEqual(["Ada Lovelace"]);
  });

  it("decodes FCE rows and treats missing chunks as empty", async () => {
    const { catalog } = await setup(true);
    const rows = await catalog.courseFce("15122");
    expect(rows[0]).toMatchObject({
      term: "F26",
      instructor: "LOVELACE, ADA",
      hours: 10.5,
      mini: null,
    });
    expect(await catalog.courseFce("21127")).toEqual([]);
    expect(await catalog.fceCoursesIndex()).toEqual(["15"]);
    expect(await catalog.courseDetails("15122")).toEqual({
      description: "Imperative programming.",
    });
    expect(await catalog.courseDetails("21127")).toBeNull();
    expect(await catalog.courseSyllabi("15122")).toEqual([]);
    expect(await catalog.finals("S27")).toBeNull();
    expect(await catalog.teaching()).toBeNull();
    expect((await catalog.offerings())?.byId.get("15122")?.terms).toEqual(["S27"]);
  });

  it("forgets failures, so the next call tries again", async () => {
    const { server, catalog } = await setup(false);
    server.fail("GET", /\/term\/S27$/u, "network");
    const failed = await settle(catalog.term("S27"));
    expect(failed.status).toBe("error");
    expect((await catalog.term("S27")).meta.id).toBe("S27");
  });

  it("rejects with ApiError for documents that don't exist", async () => {
    const { catalog } = await setup(false);
    const error: unknown = await catalog.term("F99").catch((e: unknown) => e);
    expect(error).toBeInstanceOf(ApiError);
    expect((error as ApiError).status).toBe(404);
  });
});
