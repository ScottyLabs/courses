# 13. Use API-first DuckDB browser catalog reads

Date: 2026-10-16

## Status

Superseded by [decision 16](0016-serve-the-catalog-as-json-documents.md). Previously accepted for the frontend at the project owner's request.

Supersedes decision 5's published catalog format, decision 6's browser query
implementation, and decision 7's SQLite-specific loading mechanism. Amends
decision 3's requirement to download the whole catalog before the first search.
The server database and query implementation remain the backend owner's choice.

## Context

The frontend must remain usable while its local catalog loads. The requested
browser engine is DuckDB WASM. Current server/catalog-writing work is separate
from the frontend, so a documented boundary is needed between the two.

## Decision

Components use one data repository. Initial reads use the JSON API; after
authentication, the repository loads a compatible DuckDB file in a worker and
uses SQL for covered catalog queries once ready. Unsupported queries and local
failures fall back to the API. Personal data and all mutations remain on the
API. The [browser catalog contract](../api/duckdb-catalog.md) defines the file's
SQL projection, completeness metadata, versioning, and caching behavior.

## Consequences

- The first search does not wait for the WASM engine or database download.
- Each tab holds its own read-only catalog, cached by version/access scope in OPFS.
- The selected WASM bundle adds roughly 8-9 MB compressed plus the database.
- API and local query semantics must be kept aligned; this implementation does
  not share Diesel queries between Rust and the browser or implement SQLite FTS5.
- Local reads require authenticated startup and periodic session verification;
  offline authenticated access and live snapshot synchronization are not implemented.
- The backend owner must publish the documented DuckDB projection. The current
  catalog writer is still a stub; frontend fallback does not implement the API.
