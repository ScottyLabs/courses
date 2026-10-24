// Real catalog documents for the few tests that need them, read from $CATALOG_DATA_DIR (or $MOCK_CATALOG_DIR): a
// directory of Courseweave-format files such as S27.json, catalog/index.json and programs/<auditId>.json. Catalog data
// isn't committed, so those tests are skipped without it. Only tests import this module.
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import process from "node:process";

const dir = process.env.CATALOG_DATA_DIR ?? process.env.MOCK_CATALOG_DIR ?? null;

/** True when every path exists under the data directory. */
export function haveData(...paths: string[]): boolean {
  return dir !== null && paths.every((p) => existsSync(join(dir, p)));
}

/** Parses a JSON file under the data directory. Check haveData() first. The caller names the document's type. */
// oxlint-disable-next-line typescript/no-unnecessary-type-parameters -- T is the caller's claim about a fixture file
export function readData<T>(path: string): T {
  if (dir === null) throw new Error("Set CATALOG_DATA_DIR to read catalog data in tests.");
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- fixtures are trusted to match the type asked for
  return JSON.parse(readFileSync(join(dir, path), "utf8")) as T;
}
