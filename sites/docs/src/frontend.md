> The current implementation uses SQLite WASM. See [the maintainer guide](maintaining.md) and [backend handoff](api/sqlite-backend.md); older DuckDB references below describe the original design.

# Frontend implementation notes

The SvelteKit application lives in `sites/web`. Its layout follows the
[Courses Page Final Design](https://www.figma.com/design/PMQd9pWEW4QLVfUjvG4krY/CMUCourses-Redesign--summer-25-?node-id=498-18739),
[Schedule Page Final Design](https://www.figma.com/design/PMQd9pWEW4QLVfUjvG4krY/CMUCourses-Redesign--summer-25-?node-id=498-18286),
[Final Schedule](https://www.figma.com/design/PMQd9pWEW4QLVfUjvG4krY/CMUCourses-Redesign--summer-25-?node-id=1282-16581),
and [mobile courses frame](https://www.figma.com/design/PMQd9pWEW4QLVfUjvG4krY/CMUCourses-Redesign--summer-25-?node-id=595-25268).
The feature inventory came from the
[previous CMU Courses app](https://github.com/Scottylabs/cmucourses): search,
filters, FCEs, course information, saved courses, schedules, instructors,
gen eds, and finals.

## Authentication

At startup, the frontend requests `GET /api/me` with the same-origin session
cookie. `401` means signed out, while a network or server error shows an
account-unavailable state with retry. Sign-in and sign-out are ordinary browser
navigations to the already implemented `/auth/login` and `/auth/logout` OIDC
routes. There is no browser-managed token or password form. The backend must
authorize every protected JSON field and schedule mutation; the visible state
in the frontend is not an authorization boundary.

## Local schedule draft

An anonymous schedule draft is stored under `cmucourses-draft` in localStorage.
It is not sent to the server until the user signs in and clicks **Save
schedule**. The draft holds course codes and selected section IDs. With section
details available, the schedule page shows a weekday view and marks overlapping
meetings. Saved schedules use the JSON endpoints in the [frontend API
contract](./api/frontend-contract.md). Copying a share link saves the schedule,
requests a `share_id`, and copies its read-only URL. Opening the link displays
the original read-only, and the first edit forks a local draft. The share
endpoint is documented but still requires backend implementation. Finals
can be exported as an `.ics` file from the returned exam data. The Figma finals
upload control posts a SIO `.ics` export to the documented import endpoint.

When the catalog JSON API is unavailable, the course screen shows two clearly
marked preview courses so the Figma layout remains reviewable. These are local
frontend fixtures; they are replaced by API results as soon as the endpoint
responds. Saved course stars are stored locally under `cmucourses-saved`.
The profile's Friends panel can request an invite link or add an Andrew ID;
those controls use the documented friend endpoints and show the API error until
the backend implements them.

## API and local catalog abstraction

The UI calls `data` in `src/lib/data.ts`. Catalog reads use the API during
background loading, then a read-only DuckDB WASM worker once ready. Personal
data and writes always use the API. Unsupported local datasets/terms and local
failures fall back to HTTP. The [browser DuckDB contract](./api/duckdb-catalog.md)
defines publication endpoints, required SQL tables/views, coverage, and caching.
The frontend does not create the production database. Sign-out disposes the
local provider and best-effort clears its OPFS cache.

## Running and validation

From `sites/web`, run `deno task dev`, `deno task test`, `deno task check`, and `deno task build`.
The current Rust web API provides only identity and binary catalog routes; the
new JSON catalog and user-data routes are specified in the API contract. The
frontend displays an explicit unavailable state for missing endpoints, so a
fully populated UI needs that backend work.

The pinned `svelte-check@4.4.5` currently infers every template `{#each}` item
as `unknown` in this workspace despite typed arrays. The page has a temporary
`@ts-nocheck` comment for that checker issue; build compilation and the API
contract TypeScript still run. Remove the comment after upgrading or fixing
the checker, then rerun `deno task check`.
