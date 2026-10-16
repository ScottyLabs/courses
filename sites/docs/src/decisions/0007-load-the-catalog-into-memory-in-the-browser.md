# 7. Load the catalog into memory in the browser

Date: 2026-10-14

## Status

The SQLite-specific mechanism is superseded by
[decision 13](0013-use-api-first-duckdb-browser-reads.md); loading a catalog into
each browser tab's memory is retained.

## Context

SQLite in the browser can either hold the whole catalog in memory or read it from storage as queries need it.

- Loading it into WebAssembly memory with `sqlite3_deserialize` takes one full read of the file before the first query, but no query touches storage after that, and it works in any tab
- Reading it from [OPFS](https://developer.mozilla.org/en-US/docs/Web/API/File_System_API/Origin_private_file_system) through one of SQLite's OPFS VFSs opens instantly and keeps only a page cache in memory, but browsers can't memory-map files, so any query that touches uncached pages waits on reads from storage
  - Also, it only works in a dedicated Web Worker, and using it from several tabs needs either COOP/COEP headers or one tab answering queries for the others

## Decision

We'll load the whole catalog into memory with `sqlite3_deserialize`.

## Consequences

- Each open tab holds its own copy of the catalog in memory
- The first query waits for the whole catalog to be read, which takes longer as the catalog grows
