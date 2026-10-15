# 11. Version raw scrape output by format

Date: 2026-10-15

## Status

Accepted

## Context

The build job reads the raw output of the last scrape. If a change to the scraper's output format is merged, the builder has to read output in the old format until someone runs a new scrape.

To version raw output by format, we need an identifier that changes only when the output format changes.

## Decision

- The scraper's output format is defined entirely by Rust structs. They derive a JSON Schema ([schemars](https://github.com/GREsau/schemars)), committed as `raw-format.json`, and CI fails if it's out of date
- The raw format hash is the hash of `raw-format.json`, and each scrape uploads its raw output under `raw-<hash>/`
- The build job only reads raw output whose hash matches the format compiled into it, and fails if there is none

## Consequences

- The builder never has to read an old format
- Edits that don't change the output, like comments or refactors, don't change the hash
- A pull request that changes the scraper's output format can't merge until someone runs a scrape on the pull request's branch
- A change to the output format shows up as a diff to `raw-format.json` in the pull request
