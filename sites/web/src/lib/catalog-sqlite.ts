import { browser } from "$app/environment";
import type { BindingSpec, Database } from "@sqlite.org/sqlite-wasm";

let database: Promise<Database> | undefined;

async function openCatalog(): Promise<Database> {
  const [sqlite, response] = await Promise.all([
    import("@sqlite.org/sqlite-wasm").then(({ default: init }) => init()),
    // This endpoint also serves member data when sent a session cookie.
    fetch("/api/catalog/binary", { credentials: "omit", cache: "no-store" }),
  ]);
  if (!response.ok) {
    throw new Error(`Catalog download failed (HTTP ${response.status})`);
  }

  // Fetch decodes the endpoint's gzip Content-Encoding.
  const bytes = await response.arrayBuffer();
  const { capi, oo1, wasm } = sqlite;
  const db = new oo1.DB(":memory:");
  try {
    const data = wasm.allocFromTypedArray(bytes);
    // With FREEONCLOSE, SQLite also frees the buffer if deserialization fails.
    db.checkRc(
      capi.sqlite3_deserialize(
        db,
        "main",
        data,
        bytes.byteLength,
        bytes.byteLength,
        capi.SQLITE_DESERIALIZE_FREEONCLOSE | capi.SQLITE_DESERIALIZE_READONLY,
      ),
    );
    return db;
  } catch (error) {
    db.close();
    throw error;
  }
}

/** Run a parameterized query against the public catalog in the browser. */
export async function queryCatalog(sql: string, bind?: BindingSpec) {
  if (!browser) {
    throw new Error("The SQLite catalog is only available in the browser");
  }

  database ??= openCatalog().catch((error: unknown) => {
    database = undefined;
    throw error;
  });
  return (await database).selectObjects(sql, bind);
}
