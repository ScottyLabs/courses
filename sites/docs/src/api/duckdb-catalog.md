# Browser DuckDB catalog

The frontend uses `@duckdb/duckdb-wasm` for local catalog queries. This page
specifies the downloadable database you will build. No production database,
catalog publisher, or backend handlers are implemented here. Durable user
records are specified separately in [database requirements](./database-requirements.md).

## How to understand the data layer

Components call `data` in `sites/web/src/lib/data.ts`. That repository chooses
between the HTTP adapter (`contracts.ts`) and the read-only SQL adapter
(`duckdb-catalog.ts`); components do not choose a source themselves.

1. Initial reads use the JSON API immediately.
2. A successful identity request starts the local provider in the background.
   Signed-out clients continue using the API.
3. The provider checks the catalog version, loads an OPFS-cached file or
   downloads the database, then opens it read-only in a DuckDB WASM worker.
4. After schema validation, covered course, term, instructor, and gen ed
   queries use SQL. The current screen refreshes through the same repository.
5. An uncovered dataset/term, missing detail record, initialization failure,
   or SQL error falls back to the API. Storage/quota failures permit an
   in-memory database without persistent caching.

Authentication, schedules, friends, finals, shares, and all writes continue
using the API. DuckDB is a catalog snapshot, not a synchronized copy of user
data. The active snapshot stays unchanged for that session; a subsequent
startup checks the version again. If you need live catalog refresh, add an
explicit refresh policy to the repository.

## Download contract

| Route | Required behavior |
| --- | --- |
| `GET /api/catalog/version` | Existing JSON shape `{ hash, bytes }`. Change `hash` whenever database contents or access scope change. Require the appropriate CMU session. |
| `GET /api/catalog/binary` | A checkpointed, self-contained **DuckDB database file**, optionally gzip-compressed. The `.bin` route/object name does not mean Rust struct serialization. Require the same access as the version route. |

Use `Cache-Control: private` for protected files. The existing web API's binary
handler currently emits public cache headers; change that before publishing
protected data. Scope and redact the database on the server: recipients can
read every byte. Include no student profile records, credentials, friendship
records, or uploaded personal calendars.

The browser requests `cache: no-store`, manages its own OPFS copy, handles
HTTP-decoded or raw gzip bytes, and rechecks the version after download to
avoid caching a file under the wrong revision. Compressed and decoded files
are limited to 128 MiB. Publish artifacts atomically. Invalid database/schema
files are removed from the cache and the API remains available.

Build the file with a DuckDB storage version supported by the pinned
`@duckdb/duckdb-wasm@1.33.1-dev57.0` package. Verify it with the actual WASM
reader when upgrading either producer or reader. Checkpoint and close the
writer before publishing; the database must not require a separate WAL.

## SQL projection, version 1

Expose the following tables or equivalent views in the default schema. Your
server database can use a different normalized schema and generate this
projection for publication. This SQL documents the interface; the frontend
opens the supplied file read-only and executes no schema migrations.

```sql
CREATE TABLE catalog_metadata (
    schema_version INTEGER NOT NULL
); -- Exactly one row with schema_version = 1.

CREATE TABLE catalog_coverage (
    dataset VARCHAR NOT NULL,
    term_id VARCHAR NOT NULL,
    PRIMARY KEY (dataset, term_id)
);

CREATE TABLE catalog_terms (
    id VARCHAR PRIMARY KEY,
    sort_order INTEGER NOT NULL,
    payload_json VARCHAR NOT NULL
);

CREATE TABLE catalog_courses (
    term_id VARCHAR NOT NULL,
    code VARCHAR NOT NULL,
    name VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    department VARCHAR NOT NULL,
    level INTEGER,
    units DOUBLE NOT NULL,
    offered_in VARCHAR[] NOT NULL,
    overall_course DOUBLE,
    payload_json VARCHAR NOT NULL,
    PRIMARY KEY (term_id, code)
);

CREATE TABLE catalog_instructors (
    id VARCHAR PRIMARY KEY,
    name VARCHAR NOT NULL,
    department VARCHAR,
    payload_json VARCHAR NOT NULL
);

CREATE TABLE catalog_geneds (
    id VARCHAR PRIMARY KEY,
    payload_json VARCHAR NOT NULL
);
```

Each `payload_json` is a serialized `Term`, `Course`, `Instructor`, or `Gened`
object matching `contracts.ts` and the [JSON API contract](./frontend-contract.md).
Keep nested arrays (sections/meetings, syllabi, tags, FCE summaries) in that
object. Filter columns must agree with the payload. Required arrays must be
present even if empty. Preserve nullable/unknown scores and enrollment rather
than replacing them with zero.

A covered course set must contain complete list/detail payloads for its term,
including permitted sections, all meeting blocks, requirements, syllabi,
instructors, and FCEs. Personal prerequisite satisfaction is not part of this
shared file. An omitted optional value means genuinely unavailable data,
not permission for the frontend to invent it.

### Coverage

- Dataset names are `terms`, `courses`, `instructors`, and `geneds`. A coverage
  row declares the dataset complete. No row means API fallback; a covered
  empty dataset is a real empty result.
- Use `term_id = ''` for unfiltered data. Terms, instructors, and gen ed
  coverage always use this empty string.
- A courses row for `''` covers unfiltered requests only. A row for `F26`,
  for example, covers that exact term. Use the same IDs in `catalog_terms`,
  `catalog_courses`, coverage, and the JSON API.
- For every covered term, include at most one course row per canonical code.
  All projection tables must exist, even if empty/uncovered, for validation.

Example:

```sql
INSERT INTO catalog_coverage VALUES
    ('terms', ''), ('courses', ''), ('courses', 'F26'),
    ('instructors', ''), ('geneds', '');
```

### Query semantics

Course search matches code/name/description case-insensitively and also
accepts dashless codes. Department matches its display name or a numeric
course-code prefix. Level is a numeric bucket (`100`, `200`, etc.); `500`
means >= 500. Units are numeric. `offered_in` is an array of lower-case
`fall`, `spring`, and/or `summer`. A null `overall_course` never passes a
minimum FCE filter. Rich keyword/relevance search needs a coordinated schema
and adapter extension; it is not implemented by this projection.

Courses sort by canonical code with page size capped at 100. Instructors sort
by name/ID. Terms sort by ascending `sort_order`, then ID; give the newest
relevant term the smallest order. The `all`, `fces`, `info`, and `sections`
views use the same complete course object. Keep API semantics aligned or omit
coverage until that query can be answered correctly locally.

## Cache and runtime behavior

OPFS cache keys hash the user's subject/role flags and the catalog version.
Startup must authenticate and verify the current server version before reuse.
Local reads recheck identity after 60 seconds of activity. Failed checks,
account/role changes, and sign-out discard the worker and ignore stale results.
Sign-out best-effort removes this feature's OPFS files. This is not offline
authentication; logging out cannot make already downloaded bytes unreadable.

WASM and worker assets come from the pinned package and are served from the
app's origin. The provider is dynamically imported; only the selected MVP or
exception-handling WASM bundle is fetched, after catalog access succeeds.
No public CDN is used. Serve WASM as `application/wasm` and workers as JavaScript.
CSP must allow same-origin workers and WASM compilation. The threaded COI
bundle is not enabled, so this setup does not require cross-origin isolation.
Queries run in the worker and user values are bound parameters.

## Verification

From `sites/web`, run `deno task test`, `deno task check`, and `deno task build`.
Tests cover handoff/fallback, session changes, gzip/version handling, and real
WASM SQL filters/pagination. `deno task test:browser` starts a standalone test
server; open `/tests/duckdb-browser.html` on its displayed address to verify
browser workers, SQL queries, and the API handoff. It uses a synthetic
fixture and mock API responses within that tab, and is not included in the
built app or your production database. End-to-end file compatibility and OPFS
reuse must also be verified with your published database artifact; that file
does not exist in this repository yet.

Runtime setup follows the official DuckDB
[Vite instantiation guidance](https://duckdb.org/docs/preview/clients/wasm/instantiation)
and [query API](https://duckdb.org/docs/preview/clients/wasm/query).
