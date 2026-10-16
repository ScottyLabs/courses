import { createDuckDB } from "../src/lib/duckdb-runtime";
import { createDuckDBCatalog } from "../src/lib/duckdb-catalog";
import { createDataClient } from "../src/lib/data-client";
import { api } from "../src/lib/contracts";

const output = document.querySelector<HTMLPreElement>("#result")!;
let client: ReturnType<typeof createDataClient> | undefined;

async function run() {
  let apiReads = 0;
  client = createDataClient({
    ...api,
    me: async () => ({
      subject: "synthetic-test",
      name: "Synthetic user",
      can_create: false,
      is_admin: false,
    }),
    courses: async () => {
      apiReads++;
      return { items: [], total: 7, page: 1, page_size: 12 };
    },
  }, async (_, signal) => {
    const db = await createDuckDB();
    const dispose = () => {
      void db.terminate().catch(() => {});
    };
    signal.addEventListener("abort", dispose, { once: true });
    try {
      await db.open({ path: ":memory:" });
      const connection = await db.connect();
      await connection.query(`
        CREATE TABLE catalog_metadata(schema_version INTEGER); INSERT INTO catalog_metadata VALUES (1);
        CREATE TABLE catalog_coverage(dataset VARCHAR, term_id VARCHAR); INSERT INTO catalog_coverage VALUES ('courses', '');
        CREATE TABLE catalog_terms(id VARCHAR, sort_order INTEGER, payload_json VARCHAR);
        CREATE TABLE catalog_courses(term_id VARCHAR, code VARCHAR, name VARCHAR, description VARCHAR, department VARCHAR, level INTEGER, units DOUBLE, offered_in VARCHAR[], overall_course DOUBLE, payload_json VARCHAR);
        INSERT INTO catalog_courses VALUES ('', '15-122', 'Synthetic Course', 'Test only', 'Computer Science', 100, 12, ['fall'], 4.5,
          '{"code":"15-122","name":"Synthetic Course","description":"Test only","department":"Computer Science","units":12,"offered_terms":[],"gened_tags":[]}');
        CREATE TABLE catalog_instructors(id VARCHAR, name VARCHAR, department VARCHAR, payload_json VARCHAR);
        CREATE TABLE catalog_geneds(id VARCHAR, payload_json VARCHAR);
      `);
      return await createDuckDBCatalog(connection, dispose);
    } catch (cause) {
      dispose();
      throw cause;
    }
  });
  const ready = new Promise<void>((resolve, reject) =>
    client!.subscribeCatalog((state) => {
      if (state.status === "ready") resolve();
      if (state.status === "unavailable") reject(new Error(state.error));
    })
  );
  await client.me();
  if ((await client.courses({})).total !== 7) {
    throw new Error("API did not serve warmup reads");
  }
  await ready;
  const result = await client.courses({ search: "Synthetic", units: 12 });
  if (
    result.total !== 1 || result.items[0].code !== "15-122" || apiReads !== 1
  ) throw new Error("SQL handoff failed");
  client.resetLocalCatalog();
  output.textContent =
    "PASS\nWASM worker started\nSynthetic catalog schema validated\nAPI served reads during warmup\nSQL served reads after readiness\nWorker disposed after reset\nProduction file/OPFS integration still requires your published DuckDB artifact";
}
run().catch((cause) => {
  output.textContent = `FAIL\n${cause instanceof Error ? cause.stack : cause}`;
})
  .finally(() => client?.resetLocalCatalog());
