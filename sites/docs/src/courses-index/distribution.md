# Catalog distribution

The `Scrape and Publish Catalog` workflow uploads `catalog.bin` to `s3://cdn-courses/catalog.bin`. The governance `cdn` feature provisions the `cdn-courses` bucket and sets its `CDN_*` secrets for the `dev`, `preview`, `staging`, and `prod` profiles, so every deployment reads the same object. `courses-api` and `courses-web-api` poll its ETag every `POLL_INTERVAL` seconds (default 300) and swap in the new catalog when it changes.

`courses-web-api` serves the binary at `/api/catalog/binary` and its ETag at `/api/catalog/version`. The SPA caches the binary in OPFS keyed on that ETag, so browsers download each catalog once.

When `CATALOG_UPSTREAM_URL` is set, `courses-web-api` proxies `/api/catalog/*` to that origin instead of reading S3.
