import { test } from "node:test";
import { deepEqual, equal } from "node:assert/strict";
import { createRequire } from "node:module";
import * as duckdb from "@duckdb/duckdb-wasm/blocking";
import type { AsyncDuckDBConnection } from "@duckdb/duckdb-wasm";
import { createDuckDBCatalog } from "./duckdb-catalog.ts";

test("DuckDB queries apply filters, pagination, term coverage, and bound search parameters", async () => {
  const require = createRequire(import.meta.url);
  const db = await duckdb.createDuckDB(
    {
      mvp: {
        mainModule: require.resolve("@duckdb/duckdb-wasm/dist/duckdb-mvp.wasm"),
        mainWorker: "",
      },
    },
    new duckdb.VoidLogger(),
    duckdb.NODE_RUNTIME,
  );
  await db.instantiate(() => {});
  db.open({ path: ":memory:" });
  const connection = db.connect();
  try {
    connection.query(`
      CREATE TABLE catalog_metadata(schema_version INTEGER); INSERT INTO catalog_metadata VALUES (1);
      CREATE TABLE catalog_coverage(dataset VARCHAR, term_id VARCHAR);
      INSERT INTO catalog_coverage VALUES ('courses', ''), ('courses', 'F26'), ('terms', ''), ('instructors', ''), ('geneds', '');
      CREATE TABLE catalog_terms(id VARCHAR, sort_order INTEGER, payload_json VARCHAR);
      INSERT INTO catalog_terms VALUES ('F26', 0, '{"id":"F26","label":"Fall 2026"}');
      CREATE TABLE catalog_courses(term_id VARCHAR, code VARCHAR, name VARCHAR, description VARCHAR, department VARCHAR, level INTEGER, units DOUBLE, offered_in VARCHAR[], overall_course DOUBLE, payload_json VARCHAR);
      CREATE TABLE catalog_instructors(id VARCHAR, name VARCHAR, department VARCHAR, payload_json VARCHAR);
      INSERT INTO catalog_instructors VALUES ('p1', 'Professor Example', 'Computer Science', '{"id":"p1","name":"Professor Example","courses":["15-122"]}');
      CREATE TABLE catalog_geneds(id VARCHAR, payload_json VARCHAR);
      INSERT INTO catalog_geneds VALUES ('g1', '{"id":"g1","name":"Example","courses":["15-122"]}');
    `);
    const insert = connection.prepare(
      "INSERT INTO catalog_courses VALUES (?, ?, ?, ?, ?, ?, ?, ['fall'], ?, ?)",
    );
    try {
      for (const term of ["", "F26"]) {
        for (
          const [code, level, units, score] of [["15-122", 100, 12, 4.5], [
            "15-251",
            200,
            12,
            4,
          ], ["15-513", 500, 6, null]] as const
        ) {
          const course = {
            code,
            name: `Course ${code}`,
            description: "Algorithms and computation",
            department: "Computer Science",
            units,
            offered_terms: ["F26"],
            gened_tags: [],
          };
          insert.query(
            term,
            code,
            course.name,
            course.description,
            course.department,
            level,
            units,
            score,
            JSON.stringify(course),
          );
        }
      }
    } finally {
      insert.close();
    }
    // The production adapter uses asynchronous worker connections; wrap native WASM for SQL tests.
    const asyncConnection = {
      prepare: async (sql: string) => {
        const statement = connection.prepare(sql);
        return {
          query: async (...params: unknown[]) => statement.query(...params),
          close: async () => statement.close(),
        };
      },
    } as Pick<AsyncDuckDBConnection, "prepare">;
    const catalog = await createDuckDBCatalog(asyncConnection, () => {});
    const filtered = await catalog.courses({
      term: "F26",
      search: "algorithms",
      department: "15",
      units: 12,
      offered_in: "fall",
      min_fce: 4.2,
    });
    equal(filtered?.total, 1);
    equal(filtered?.items[0].code, "15-122");
    equal((await catalog.courses({ search: "15122" }))?.total, 1);
    equal((await catalog.courses({ level: "500" }))?.items[0].code, "15-513");
    const second = await catalog.courses({ page: 2, page_size: 1 });
    equal(second?.total, 3);
    equal(second?.items[0].code, "15-251");
    equal((await catalog.courses({ search: "' OR 1=1 --" }))?.total, 0);
    equal(await catalog.courses({ term: "S27" }), undefined);
    equal((await catalog.course("15-122", "F26"))?.units, 12);
    deepEqual(await catalog.terms(), [{ id: "F26", label: "Fall 2026" }]);
    equal((await catalog.instructors("example"))?.items[0].id, "p1");
    equal((await catalog.instructor("p1"))?.name, "Professor Example");
    equal((await catalog.geneds())?.[0].id, "g1");
  } finally {
    connection.close();
    db.reset();
  }
});
