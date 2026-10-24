# 10. Build the catalog separately from scraping

Date: 2026-10-14

## Status

Accepted; amended by [decision 15](0015-build-the-catalog-in-rust.md), which builds one `catalog.sqlite`

## Context

We want each release of the website and the public course API to read a catalog built by the same code. Scraping the course data needs a student's [Stellic](https://academicaudit.andrew.cmu.edu) cookie and [Canvas](https://canvas.cmu.edu) token, so it only happens when someone manually starts it.

## Decision

- Each scrape uploads its raw output to object storage
- A separate build job turns the latest raw output into a catalog after every scrape and on every push to a branch we deploy
- Each catalog is named after a hash of its schema's SQL, including the FTS5 table and indexes, as `catalog-<hash>.sqlite`
  - We don't use a version number raised by hand, since two branches could raise it to the same value with different tables
  - The web backend and the public course API each compute the hash of the schema compiled into them and load the catalog with that name. The web backend's `/api/catalog/version` endpoint reports this hash as the catalog format identifier
- The build is deterministic, so the same raw output and code always produce the same bytes
  - Rows are inserted in a fixed order, sorted rather than taken from hash map iteration
  - The gzip header has no timestamp or filename
  - CI builds the catalog twice from the same raw output and fails if the two files differ

## Consequences

- Changing the tables doesn't need a new scrape, since the push that changes them also builds a catalog for the new schema from the latest raw output
- A rebuild that changes nothing keeps the same ETag, so users keep their cached catalog (except after upgrading SQLite)
- Old catalogs accumulate in object storage and have to be deleted
  - We can't keep only the newest catalog, since `main`, `staging`, `dev`, and each pull request preview can be on a different schema
