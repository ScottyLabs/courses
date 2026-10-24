# 16. Serve the catalog to the browser as JSON documents

Date: 2026-10-24

## Status

Accepted. Supersedes [decision 13](0013-use-api-first-duckdb-browser-reads.md) and what remained of
[decision 7](0007-load-the-catalog-into-memory-in-the-browser.md). Amends [decision 8](0008-cache-the-catalog-in-opfs.md):
the browser's HTTP cache, keyed by ETag, replaces the OPFS copy. [Decision 3](0003-query-the-catalog-in-the-browser.md)
still holds: the website queries the catalog in the browser.

## Context

The browser has been loading the catalog as a SQLite database: a WebAssembly build of SQLite plus a 4-7 MB
database, with the JSON API answering queries until both arrive. Every query exists twice, in Rust for the API and in
TypeScript for the worker, and the two have to agree.

Courseweave's features need the whole semester in memory: ranked search, filtering by requirement or by what fits a
schedule, conflict checks, requisite checks against a student's record and degree audits. Its data is split into
JSON documents by what a screen needs. A semester's sections are about 170 KB gzipped and the course catalog about
250 KB, less than SQLite's WebAssembly build alone.

## Decision

- The catalog builder ([decision 15](0015-build-the-catalog-in-rust.md)) writes the website's data as named JSON
  documents in `catalog.sqlite`, in Courseweave's formats: `terms`, `term/<id>`, `catalog/index`,
  `catalog/details/<prefix>`, `offerings`, `teaching`, `fce/*`, `programs/*`, `syllabi/<prefix>` and `finals/<id>`.
- `GET /api/catalog/<name>` serves a document's stored gzip bytes with its ETag and `Cache-Control: private, no-cache`, and answers `If-None-Match` with 304. The browser revalidates on each load and downloads a document
  again only when it changed.
- Each document has a public version and may have a member version, and the server picks the version by the caller's
  sign-in. A document with only a member version returns 401 to signed-out callers. Public versions leave out
  instructors, meeting rooms, FCEs, syllabi and Stellic degree programs, as the SQLite catalog did.
- The frontend loads the documents a screen needs and runs Courseweave's TypeScript modules over them. SQLite's
  WebAssembly build, the worker and the OPFS cache are removed.
- The JSON query endpoints (`/api/courses` and the like) stay, served from the relational tables, for the public
  course API and other clients. The website doesn't use them.

## Consequences

- The first search waits for one semester document of about 170 KB gzipped, with no WebAssembly to load
- Queries exist once, in TypeScript; the server's endpoints are for API clients and can be simpler
- Each member/public split is decided by the builder, which writes both versions, and enforced by the server, which
  picks one
- Data the browser has downloaded can't be retracted after sign-out, as before
- Large documents are split by course-number prefix (`catalog/details/15`, `fce/courses/15`), so a course page
  downloads only its department's slice
