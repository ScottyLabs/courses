import { test } from "node:test";
import { equal, deepEqual } from "node:assert/strict";
import initSqlJs from "sql.js";
import { createSQLiteCatalog } from "./sqlite-catalog.ts";

test("SQLite uses bound search parameters, term scope, and exact seasonal filters", async () => {
  const SQL = await initSqlJs();
  const db = new SQL.Database();
  db.run(`CREATE TABLE catalog_metadata(schema_version); INSERT INTO catalog_metadata VALUES(1);
    CREATE TABLE catalog_coverage(dataset,term_id); INSERT INTO catalog_coverage VALUES('courses','F26');
    CREATE TABLE catalog_terms(id,sort_order,payload_json);
    CREATE TABLE catalog_courses(term_id,code,name,description,department,level,units,offered_in,overall_course,payload_json);
    CREATE TABLE catalog_instructors(id,name,department,payload_json);
    CREATE TABLE catalog_geneds(id,payload_json);`);
  const c = {
    code: "15-122",
    name: "Computation",
    description: "C and contracts",
    units: 12,
    department: "Computer Science",
    offered_terms: ["F26"],
    gened_tags: [],
  };
  db.run("INSERT INTO catalog_courses VALUES(?,?,?,?,?,?,?,?,?,?)", [
    "F26",
    c.code,
    c.name,
    c.description,
    c.department,
    100,
    12,
    "fall|spring",
    4,
    JSON.stringify(c),
  ]);
  const catalog = await createSQLiteCatalog(
    {
      async query(sql, params = []) {
        const statement = db.prepare(sql);
        try {
          statement.bind(params);
          const rows = [];
          while (statement.step()) rows.push(statement.getAsObject());
          return rows;
        } finally {
          statement.free();
        }
      },
    },
    () => db.close(),
  );
  deepEqual(
    (await catalog.courses({ term: "F26", search: "15122", offered_in: "fall", department: "15" }))
      ?.items,
    [c],
  );
  equal((await catalog.courses({ term: "F26", search: "' OR 1=1 --" }))?.total, 0);
  equal((await catalog.courses({ term: "F26", offered_in: "all" }))?.total, 0);
  equal(await catalog.courses({ term: "S26" }), undefined);
  catalog.dispose();
});
