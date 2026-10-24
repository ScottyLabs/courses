import initSqlJs, { type Database } from "sql.js";
import wasmUrl from "sql.js/dist/sql-wasm.wasm?url";
let db: Database | undefined;
self.onmessage = async ({ data: { id, sql, params, bytes } }) => {
  try {
    if (bytes) {
      const SQL = await initSqlJs({ locateFile: () => wasmUrl });
      db = new SQL.Database(bytes);
      db.run("PRAGMA query_only=ON");
      self.postMessage({ id, result: [] });
      return;
    }
    if (!db) throw new Error("SQLite is not ready");
    const statement = db.prepare(sql);
    try {
      statement.bind(params ?? []);
      const result = [];
      while (statement.step()) result.push(statement.getAsObject());
      self.postMessage({ id, result });
    } finally { statement.free(); }
  } catch (cause) {
    self.postMessage({ id, error: cause instanceof Error ? cause.message : String(cause) });
  }
};
