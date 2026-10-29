# Overview

> Historical pipeline overview. Both API crates are now skeletons; catalog endpoints return 501. See the [current backend guide](../../docs/src/backend-development.md). Source collection and database tooling remain.

The CMU Courses project has three layers:

1. `courses-scraper` pulls course information, sections, program audits, and syllabi from Stellic and Canvas. Output lands under `exported/` as JSON files keyed by course code, program, and term.

1. `courses-index` loads `exported/` and `data/fces.csv` into the course catalog that the scrape workflow publishes as `catalog.bin`.

1. Two APIs consume the catalog: `courses-api` is the public REST surface, and `courses-web-api` is the web app's backend, which serves the SPA bundle and the catalog to the browser on a single origin.

The scraper section below documents the per-source collection logic.
