# Frontend API contract

> Implementation update (2026-10-23): core catalog, auth, schedule, saved-course,
> and requirements routes now exist. See [SQLite backend](./sqlite-backend.md)
> for the current behavior and deferred routes. Older DuckDB notes below are historical.

The Svelte frontend calls same-origin JSON routes defined in
`sites/web/src/lib/contracts.ts`. **This document is a backend implementation
request; these routes have not been implemented here.** The current Rust web API
already owns the OIDC session and serves the frontend, so it should be the
browser-facing origin. No bearer token or Andrew credentials should be stored
in browser storage.

See [database requirements](./database-requirements.md) for durable records,
stable catalog references, ownership rules, indexes, and source-data gaps.
The [browser DuckDB contract](./duckdb-catalog.md) specifies the read-only
database publication format used after the initial API requests.

## Current contracts

The implementation has superseded this original endpoint wishlist. Use:

- [Catalog data and releases](data.md)
- [Personal API and consent](user.md)
- [Backend and browser handoff](sqlite-backend.md)
- [Feature status](../features.md)
- `sites/web/src/lib/contracts.ts` for wire types

The browser uses SQLite WASM, not DuckDB. There are no fake preview courses. Anonymous weekly times are public; instructor identities, FCEs and locations remain restricted. All personal mutations stay server-side.
