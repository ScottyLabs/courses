import { openLocalCatalog } from "../src/lib/local-catalog";
import { createDataClient } from "../src/lib/data-client";
import { api, ApiError } from "../src/lib/contracts";
const output = document.querySelector<HTMLPreElement>("#result")!;
let reads = 0;
const client = createDataClient({ ...api,
  me: async () => { throw new ApiError(401, "Signed out"); },
  courses: async () => { reads++; return {items:[],total:7,page:1,page_size:12}; },
}, openLocalCatalog);
async function run() {
 const ready = new Promise<void>((resolve,reject) => client.subscribeCatalog((s) => {
  if(s.status === "ready") resolve(); if(s.status === "unavailable") reject(new Error(s.error));
 }));
 await client.me().catch(() => {});
 if((await client.courses({})).total !== 7) throw new Error("API warmup failed");
 await ready;
 const result=await client.courses({search:"15-122",term:"F26"});
 if(result.items[0]?.code !== "15-122" || reads !== 1) throw new Error("SQLite handoff failed");
 if(result.items.some(c => c.fce || c.instructors?.length || c.sections?.some(s => s.instructors.length || s.meetings.some(m => m.location)))) throw new Error("Public disclosure");
 client.resetLocalCatalog();
 output.textContent="PASS\nSQLite WASM worker loaded the imported catalog\nAPI handled warmup search\nSQLite handled subsequent search\nPublic result excludes restricted fields\nWorker disposed";
}
run().catch(e => {output.textContent=`FAIL\n${e.stack ?? e}`;}).finally(() => client.resetLocalCatalog());
