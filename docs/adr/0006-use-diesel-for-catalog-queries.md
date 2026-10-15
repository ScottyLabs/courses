# 6. Use Diesel for catalog queries

Date: 2026-10-14

## Status

Accepted

## Context

We need a query library for the catalog that supports SQLite, builds for `wasm32-unknown-unknown`, and is type-safe.

- [sqlx](https://github.com/transact-rs/sqlx) checks raw SQL against the database at compile time, but doesn't build for `wasm32-unknown-unknown`
- [SeaORM](https://www.sea-ql.org/SeaORM/) is type-safe, but has no documented support for `wasm32-unknown-unknown`
- [Diesel](https://diesel.rs) is type-safe, and its SQLite backend builds for `wasm32-unknown-unknown`

## Decision

We'll write the catalog's schema and queries with Diesel, in one crate that both the server and the browser build. Result structs derive [utoipa](https://github.com/juhaku/utoipa)'s `ToSchema` and [tsify](https://github.com/madonoharu/tsify)'s `Tsify`, so the public course API's OpenAPI spec and the browser's TypeScript types come from the same structs.

## Consequences

- The website and the public course API run the same query code, so they return the same results
- Changing a query's result type updates the OpenAPI spec and the browser's TypeScript types together
