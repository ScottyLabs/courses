# 29. Develop and test the website against an in-memory mock of the web API

Date: 2026-10-24

## Status

Accepted

## Context

The website was rebuilt at the same time as the web API, by different people, so for a while there was no API to run
against. Even with the API ready, running it locally takes a built `catalog.sqlite`, a user database and a sign-in
setup, which is a lot for someone working on a page. The stores' tests also need something that answers like the API:
revisions, 409s with the stored record, 401s for member documents, sign-in and sign-out.

## Decision

- `sites/web/dev/mock-server.ts` is a small, in-memory version of the API's routes. Catalog documents come from a
  directory of Courseweave-format files (`MOCK_CATALOG_DIR`, such as Courseweave's `public/data`); member documents
  answer 401 when signed out, and term documents leave out instructors and rooms. Schedules, plans, the record,
  preferences and saved courses live in memory with revisions and 409s. Friends and finals are only deep enough for the
  pages to render.
- `dev/mock-api.ts` is a Vite plugin that serves it on `/api` and `/auth` during `vite dev` when `MOCK_API=1`
  (`deno task dev:mock`). `vite.config.ts` imports it only then, and the plugin applies only to the dev server, so it is
  never part of a build.
- The stores' tests use the same mock through the real API client, with a fetch that calls it directly.

## Consequences

- Pages can be built and tried signed in or out, with real catalog data, without the Rust API
- The mock has to follow the API: when routes or bodies change (`openapi.json`), the mock changes with them, or
  tests that pass against it can hide a mismatch
- The mock checks nothing the API checks beyond what the website needs to see (it trusts every request), so it is no
  substitute for trying changes against the real API before release
