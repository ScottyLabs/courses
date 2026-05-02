# Overview

The CMU Courses project has three layers:

1. The **scraper** (`courses-scraper`) pulls course information, sections, program audits, and syllabi from Stellic and Canvas. Output lands under `exported/` as JSON files keyed by course code, program, and term.

2. The **index** (`courses-index`) loads `exported/` into a single binary catalog with text, facet, numeric, and schedule indexes baked in. The same crate compiles to native (for the build step that produces `catalog.bin`) and to wasm (for the browser query engine).

3. Two **APIs** consume the catalog: `courses-api` is the public REST surface (versioned under `/v1/`, in-memory, no database), and `courses-web-api` is the web app's backend (serves the catalog binary to the browser, hosts the SPA, future home for auth and saved schedules on top of seaORM).

The scraper section below documents the per-source collection logic. The courses index section documents the on-disk catalog format and the query engine.
