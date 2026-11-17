import assert from "node:assert/strict";
import { DatabaseSync } from "node:sqlite";
import { test } from "node:test";
import { createCatalogReader } from "../src/lib/catalog-reader.ts";

function withCatalog(
  run: (catalog: ReturnType<typeof createCatalogReader>, calls: string[]) => Promise<void>,
) {
  return async () => {
    const db = new DatabaseSync(":memory:");
    try {
      db.exec(`
        CREATE TABLE catalog_courses (
          term_id TEXT, code TEXT, name TEXT, description TEXT,
          department TEXT, level INTEGER, units REAL, offered_in TEXT, overall_course REAL,
          PRIMARY KEY (term_id, code)
        );
        INSERT INTO catalog_courses VALUES
          ('', '15-100', 'Systems', 'Introduction', 'Computer Science', 100, 9, 'fall', NULL),
          ('', '15-200', 'Systems', 'Advanced', 'Computer Science', 200, 9, 'fall|spring', 4.1),
          ('', '15-300', 'Theory', 'Systems and proofs', 'Computer Science', 300, 12, 'spring', 4.5),
          ('', '21-100', 'Systems', '', 'Mathematics', 100, 9, 'fall', NULL),
          ('F26', '15-200', 'Systems', 'Advanced', 'Computer Science', 200, 9, 'fall|spring', 4.1);
        CREATE TABLE catalog_instructors (id TEXT PRIMARY KEY, name TEXT, department TEXT);
        INSERT INTO catalog_instructors VALUES ('2', 'Ada', 'CS'), ('1', 'Ada', 'CS'), ('3', 'Grace', NULL);
        CREATE TABLE catalog_terms (id TEXT PRIMARY KEY, sort_order INTEGER, payload_json TEXT);
        INSERT INTO catalog_terms VALUES ('F26', 1, '{"label":"Fall 2026"}'), ('S27', 0, '{"label":"Spring 2027"}');
        CREATE TABLE catalog_coverage (dataset TEXT, term_id TEXT);
        INSERT INTO catalog_coverage VALUES ('courses', 'F26');
      `);
      const calls: string[] = [];
      const query = (audience: string) => (sql: string, bind: (string | number)[]) => {
        calls.push(audience);
        return Promise.resolve(db.prepare(sql).all(...bind));
      };
      await run(createCatalogReader(query("public"), query("member")), calls);
    } finally {
      db.close();
    }
  };
}

void test(
  "course search composes with filters, term scope, and stable pagination",
  withCatalog(async (catalog) => {
    const options = {
      search: " SYSTEMS ",
      department: "15",
      offeredIn: "fall",
      units: 9,
      sort: { field: "name", direction: "desc" },
      pageSize: 1,
    } as const;
    const first = await catalog.listCourses(options);
    assert.deepEqual(
      first.items.map(({ code }) => code),
      ["15-100"],
    );
    assert.equal(first.nextPage, 2);
    const second = await catalog.listCourses({
      ...options,
      page: first.nextPage,
    });
    assert.deepEqual(
      second.items.map(({ code }) => code),
      ["15-200"],
    );
    assert.equal(second.nextPage, null);
    assert.equal(second.items[0].overallCourse, null);

    const byCode = await catalog.listCourses({
      term: "F26",
      search: "15200",
      minLevel: 200,
    });
    assert.deepEqual(
      byCode.items.map(({ code }) => code),
      ["15-200"],
    );
    assert.deepEqual((await catalog.listCourses({ search: "'%" })).items, []);
    assert.deepEqual(await catalog.listCourses({ page: 100 }), {
      items: [],
      page: 100,
      nextPage: null,
    });
  }),
);

void test(
  "streams fetch on demand and abort before reading another page",
  withCatalog(async (catalog, calls) => {
    const controller = new AbortController();
    const stream = catalog.streamCourses({
      pageSize: 1,
      signal: controller.signal,
    });
    assert.equal(calls.length, 0);
    const first = await stream.next();
    assert.equal(first.done, false);
    assert.equal(first.value.items[0].code, "15-100");
    assert.equal(calls.length, 1);
    controller.abort();
    await assert.rejects(stream.next(), { name: "AbortError" });
    assert.equal(calls.length, 1);
  }),
);

void test(
  "member filters and instructors use the member reader; terms report course coverage",
  withCatalog(async (catalog, calls) => {
    const courses = await catalog.listCourses({
      audience: "member",
      minFce: 4,
      sort: { field: "overallCourse", direction: "desc" },
    });
    assert.deepEqual(
      courses.items.map(({ overallCourse }) => overallCourse),
      [4.5, 4.1],
    );
    const ids = [];
    for await (const page of catalog.streamInstructors({
      search: "ada",
      department: "cs",
      pageSize: 1,
    })) {
      ids.push(page.items[0].id);
    }
    assert.deepEqual(ids, ["1", "2"]);
    assert.deepEqual(await catalog.listTerms(), [
      { id: "S27", label: "Spring 2027", hasCourses: false },
      { id: "F26", label: "Fall 2026", hasCourses: true },
    ]);
    assert.deepEqual(calls, ["member", "member", "member", "public"]);
  }),
);

void test(
  "invalid pagination and sort options fail before querying SQLite",
  withCatalog(async (catalog, calls) => {
    await assert.rejects(catalog.listCourses({ page: 0 }), RangeError);
    await assert.rejects(catalog.listCourses({ pageSize: 101 }), RangeError);
    await assert.rejects(
      // @ts-expect-error Verify runtime validation for callers without TypeScript.
      catalog.listCourses({ sort: { field: "constructor" } }),
      TypeError,
    );
    assert.deepEqual(calls, []);
  }),
);
