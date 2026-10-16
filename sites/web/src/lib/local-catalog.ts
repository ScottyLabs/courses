import { loadCatalog } from "./catalog";
import { createDuckDB, DuckDBAccessMode } from "./duckdb-runtime";
import { createDuckDBCatalog } from "./duckdb-catalog";
import type { User } from "./contracts";
import type { LocalCatalog } from "./data-source";

export async function openLocalCatalog(
  user: User,
  signal: AbortSignal,
): Promise<LocalCatalog> {
  const scope = JSON.stringify([user.subject, user.can_create, user.is_admin]);
  // Don't download/instantiate WASM until authenticated catalog access succeeds.
  const download = await loadCatalog(scope, signal);
  signal.throwIfAborted();
  const db = await createDuckDB();
  let disposed = false;
  const dispose = () => {
    if (!disposed) {
      disposed = true;
      void db.terminate().catch(() => {});
    }
  };
  signal.addEventListener("abort", dispose, { once: true });
  try {
    signal.throwIfAborted();
    // DuckDB transfers the buffer to its worker, so retain the cache's own buffer.
    await db.registerFileBuffer("catalog.duckdb", download.bytes.slice());
    await db.open({
      path: "catalog.duckdb",
      accessMode: DuckDBAccessMode.READ_ONLY,
    });
    const connection = await db.connect();
    const local = await createDuckDBCatalog(connection, dispose);
    await download.commit();
    signal.throwIfAborted();
    return local;
  } catch (cause) {
    dispose();
    await download.invalidate();
    throw cause;
  }
}
