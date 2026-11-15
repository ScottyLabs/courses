# Browser catalog with SQLite WASM

The frontend can query a read-only copy of the public course catalog through
`$lib/catalog-sqlite`. Call the helper from a browser event handler or Svelte's
`onMount`.

```ts
import { queryCatalog } from "$lib/catalog-sqlite";

const terms = await queryCatalog("SELECT * FROM catalog_terms LIMIT ?", [10]);
```

The helper returns rows as objects. Keep SQL in application code and pass user
input as bound parameters. Components can handle a rejected query like any
other failed data request.

## Loading and caching

On the first query, Vite's SQLite WASM module and the catalog download start in
parallel. The browser decompresses the response using its gzip content encoding.
SQLite opens the resulting bytes as a read-only, in-memory database on the main
thread. Concurrent callers share the opening promise, later queries reuse the
connection, and a failed open can be retried on the next call.

The server downloads and decrypts the catalog release from S3. The frontend
uses the same-origin `/api/catalog/binary` endpoint without session cookies to
select the public snapshot. The request bypasses the HTTP cache because that
URL can also serve a member snapshot. Reloading the page requires another
download.

## Follow-up work

Add typed functions such as `searchCourses` above the query helper as screens
need them. Move query execution to a worker if catalog loading or queries
affect UI responsiveness. A dedicated public URL keyed by catalog release
would allow browser caching across page loads.
