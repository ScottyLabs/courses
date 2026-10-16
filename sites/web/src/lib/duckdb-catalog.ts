import type { AsyncDuckDBConnection } from "@duckdb/duckdb-wasm";
import type { Course, CourseQuery, Gened, Instructor, Term } from "./contracts";
import type { LocalCatalog } from "./data-source";

// SQL parameters are always bound; no user input becomes SQL syntax.
type SqlConnection = Pick<AsyncDuckDBConnection, "prepare">;
async function rows(
  connection: SqlConnection,
  sql: string,
  params: (string | number)[] = [],
) {
  const statement = await connection.prepare(sql);
  try {
    return (await statement.query(...params)).toArray();
  } finally {
    await statement.close();
  }
}

function payload<T>(row: { payload_json: string }): T {
  return JSON.parse(row.payload_json) as T;
}

export async function createDuckDBCatalog(
  connection: SqlConnection,
  dispose: () => void,
): Promise<LocalCatalog> {
  const metadata = await rows(
    connection,
    "SELECT schema_version FROM catalog_metadata",
  );
  if (metadata.length !== 1 || metadata[0].schema_version !== 1) {
    throw new Error("Unsupported DuckDB catalog schema");
  }
  const coverageRows = await rows(
    connection,
    "SELECT dataset, term_id FROM catalog_coverage",
  );
  const coverage = new Set(
    coverageRows.map((row) => `${row.dataset}:${row.term_id}`),
  );
  // Validate expected columns before switching any reads away from the API.
  const schemas = [
    "SELECT id, sort_order, payload_json FROM catalog_terms LIMIT 0",
    "SELECT term_id, code, name, description, department, level, units, offered_in, overall_course, payload_json FROM catalog_courses LIMIT 0",
    "SELECT id, name, department, payload_json FROM catalog_instructors LIMIT 0",
    "SELECT id, payload_json FROM catalog_geneds LIMIT 0",
  ];
  for (const sql of schemas) await rows(connection, sql);
  const covered = (dataset: string, term = "") =>
    coverage.has(`${dataset}:${term}`);

  return {
    async terms() {
      if (!covered("terms")) return undefined;
      return (await rows(
        connection,
        "SELECT payload_json FROM catalog_terms ORDER BY sort_order, id",
      )).map((row) => payload<Term>(row));
    },
    async courses(query: CourseQuery) {
      const term = query.term ?? "";
      if (!covered("courses", term)) return undefined;
      const where = ["term_id = ?"];
      const params: (string | number)[] = [term];
      if (query.search?.trim()) {
        where.push(
          "(contains(lower(code || ' ' || name || ' ' || description), ?) OR contains(replace(code, '-', ''), ?))",
        );
        params.push(
          query.search.trim().toLowerCase(),
          query.search.trim().replaceAll("-", ""),
        );
      }
      if (query.department) {
        where.push("(lower(department) = lower(?) OR starts_with(code, ?))");
        params.push(query.department, query.department);
      }
      if (query.level) {
        where.push(query.level === "500" ? "level >= ?" : "level = ?");
        params.push(Number(query.level));
      }
      if (query.units !== "" && query.units !== undefined) {
        where.push("units = ?");
        params.push(Number(query.units));
      }
      if (query.offered_in) {
        where.push("list_contains(offered_in, ?)");
        params.push(query.offered_in.toLowerCase());
      }
      if (query.min_fce !== "" && query.min_fce !== undefined) {
        where.push("overall_course >= ?");
        params.push(Number(query.min_fce));
      }
      const clause = where.join(" AND ");
      const page = Math.max(1, Math.trunc(query.page ?? 1));
      const pageSize = Math.max(
        1,
        Math.min(100, Math.trunc(query.page_size ?? 12)),
      );
      const count = await rows(
        connection,
        `SELECT count(*) AS total FROM catalog_courses WHERE ${clause}`,
        params,
      );
      const result = await rows(
        connection,
        `SELECT payload_json FROM catalog_courses WHERE ${clause} ORDER BY code LIMIT ? OFFSET ?`,
        [...params, pageSize, (page - 1) * pageSize],
      );
      return {
        items: result.map((row) => payload<Course>(row)),
        total: Number(count[0].total),
        page,
        page_size: pageSize,
      };
    },
    async course(code, term = "") {
      if (!covered("courses", term)) return undefined;
      const result = await rows(
        connection,
        "SELECT payload_json FROM catalog_courses WHERE code = ? AND term_id = ?",
        [code, term],
      );
      // Unknown records fall back to API for canonical 404/error behavior.
      return result.length ? payload<Course>(result[0]) : undefined;
    },
    async instructors(search) {
      if (!covered("instructors")) return undefined;
      const result = await rows(
        connection,
        "SELECT payload_json FROM catalog_instructors WHERE contains(lower(name || ' ' || coalesce(department, '')), ?) ORDER BY name, id",
        [search.trim().toLowerCase()],
      );
      return {
        items: result.map((row) => payload<Instructor>(row)),
        total: result.length,
      };
    },
    async instructor(id) {
      if (!covered("instructors")) return undefined;
      const result = await rows(
        connection,
        "SELECT payload_json FROM catalog_instructors WHERE id = ?",
        [id],
      );
      return result.length ? payload<Instructor>(result[0]) : undefined;
    },
    async geneds() {
      if (!covered("geneds")) return undefined;
      return (await rows(
        connection,
        "SELECT payload_json FROM catalog_geneds ORDER BY id",
      )).map((row) => payload<Gened>(row));
    },
    dispose,
  };
}
