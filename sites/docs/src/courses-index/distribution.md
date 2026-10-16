# Catalog distribution

The `Scrape and Publish Catalog` workflow uploads `catalog.bin` to `s3://cdn-courses/catalog.bin`. The governance `cdn` feature provisions the `cdn-courses` bucket and sets its `CDN_*` secrets for the `dev`, `preview`, `staging`, and `prod` profiles, so every deployment reads the same object. `courses-api` and `courses-web-api` poll its ETag every `POLL_INTERVAL` seconds (default 300) and swap in the new catalog when it changes.

`courses-web-api` serves the binary at `/api/catalog/binary` and its version hash at `/api/catalog/version`. The frontend now expects a compatible, checkpointed DuckDB database at the binary route and caches a validated copy in OPFS per user/access scope and hash. JSON API reads remain available while DuckDB WASM loads. The artifact writer in `courses-index` is currently unimplemented; the publisher must produce the schema in the [browser DuckDB contract](../api/duckdb-catalog.md) before local SQL reads can activate.

When `CATALOG_UPSTREAM_URL` is set, `courses-web-api` proxies `/api/catalog/*` to that origin instead of reading S3.
