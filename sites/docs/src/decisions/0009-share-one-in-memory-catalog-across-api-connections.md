# 9. Share one in-memory catalog across the public course API's connections

Date: 2026-10-14

## Status

Accepted

## Context

We want the public course API to handle many requests at once while keeping only one copy of the catalog in memory.

A Diesel connection can only be used by one thread at a time, so handling requests in parallel needs a pool of connections. An in-memory SQLite database normally belongs to the connection that opened it, though, so every connection in the pool would load its own copy.

SQLite's built-in `memdb` [VFS](https://sqlite.org/vfs.html) keeps a database in memory and lets every connection in the same process that opens it by the same name share it. The name has to start with `/`, as in `file:/catalog-<etag>?vfs=memdb`.

Deserializing bytes into a connection replaces its database with a new private one, even if the connection opened a shared `memdb` name.

## Decision

- We'll deserialize each downloaded catalog into a throwaway in-memory connection, then copy it into a shared `memdb` database named after its ETag with `VACUUM INTO 'file:/catalog-<etag>?vfs=memdb'`
- Requests are served from a pool of connections that all open that name
- When a new catalog arrives, we load it under its own name and swap in a new pool, and requests already running finish on the old one

## Consequences

- We can size the pool purely for how many queries we want to run at once
- SQLite frees a shared `memdb` database when its last connection closes, so a pool connection has to be open on the name before `VACUUM INTO` runs, and the public course API must keep at least one open afterwards
- Loading a catalog briefly holds two copies in memory, the throwaway one and the shared one
- Each connection also caches the pages it has recently read, up to SQLite's default of about 2 MB
