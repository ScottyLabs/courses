# 3. Query the catalog in the browser

Date: 2026-10-14

## Status

Accepted; the initial-load behavior is amended by
[decision 13](0013-use-api-first-duckdb-browser-reads.md).

## Context

The course data changes only a few times a semester and is small enough for a browser to download whole.

## Decision

We'll package the course data as a single catalog file and run the website's queries against it in the browser.

## Consequences

- Search is instant and works offline
  - To truly work offline, we also need the website's code cached in the browser, meaning a service worker
- Users download the whole catalog before their first search, so it can't be too large
- We won't get search analytics unless we record it separately from the website
- The shared interface around the course data has to build for `wasm32-unknown-unknown` as well as `x86_64-unknown-linux-gnu` / `aarch64-unknown-linux-gnu`
