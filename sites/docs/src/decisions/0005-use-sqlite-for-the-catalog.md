# 5. Use SQLite for the catalog

Date: 2026-10-14

## Status

Accepted

## Context

The browser must be able to execute several types of queries against the catalog file, including ranked full-text search. A pragmatic option is to use an actual embedded database that is stored as a single file and compiles to WebAssembly.

- [DuckDB](https://duckdb.org)'s browser build is over an order of magnitude larger than SQLite's, and its full-text search is a separate download without per-column weights
- [SQLite](https://sqlite.org) has FTS5 built in, with per-column weights in `bm25()`

## Decision

We'll store the catalog as a SQLite database and use FTS5 for text search.

## Consequences

- Users download SQLite's WebAssembly build along with the catalog
- The catalog is a standard SQLite file, so developers have the option to use that directly, in addition to the public course API
