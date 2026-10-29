# Backend development

The backend is intentionally a skeleton. The Svelte frontend is retained, but its server-backed catalog, schedules, saved courses, friends and finals features need implementation. Existing data is not deleted.

## What remains

- `crates/courses-web-api/src/main.rs`: executable server, route registrations, static SPA serving, tracing, body limit, OIDC login/callback/logout and sessions.
- `auth.rs`: verified CMU identity extractor and `/api/me`. Identity lookup no longer creates or updates profiles.
- `backend/mod.rs`: SQLite connections, personal schema initialization, readiness checks, shared JSON error and Origin/cache middleware.
- `catalog_release.rs`: authenticated, encrypted catalog installation and release checks.
- `scripts/`: database import, verification, publication, backup and development initialization tooling. Source scrapers and index builders remain as database population tooling in `courses-scraper` and `courses-index`.
- `devenv.nix`, service scripts, PostgreSQL setup and existing SQLite data remain. The web API currently uses SQLite; PostgreSQL is available development infrastructure.
- `courses-api`: legacy standalone server with its original endpoint paths and API docs; S3 fetching and polling have been removed. It is not the frontend backend.

## Placeholder behavior

All handlers in `backend/catalog.rs`, `personal.rs`, `friends.rs` and `finals.rs` return HTTP **501** with `{"message":"Backend endpoint not implemented"}`. They retain their original request extractor signatures and perform no feature queries or writes. Extractor rejection (for example invalid JSON), authentication middleware and the mutation Origin check can respond before the stub is called. Unknown endpoints still return 404.

`/api/health` retains catalog readiness checks: an empty development catalog is not ready and returns 503. Health does not claim product features work. `/api/me`, `/auth/login`, `/auth/callback`, `/auth/logout`, `/api/openapi.json` and `/api/scalar` retain infrastructure behavior. Web OpenAPI currently covers only the annotated infrastructure routes; the route registrations in `main.rs` are authoritative.

The legacy service retains `/api/health` (process liveness), `/api/version`, `/api/binary`, `/api/courses/{code}`, `/api/openapi.json` and `/api/scalar`. Its three catalog routes return 501 and require no S3 credentials.

## Where to implement each feature

| File under `crates/courses-web-api/src/backend` | Retained routes and methods | Work needed |
| --- | --- | --- |
| `catalog.rs` | GET `/api/catalog/version`, `/api/catalog/binary`, `/api/terms`, `/api/courses`, `/api/courses/{code}`, `/api/instructors`, `/api/instructors/{id}`, `/api/geneds`, `/api/programs`, `/api/programs/{id}` | Audience selection, parameterized queries, filters, pagination, wire formats and authorized downloads |
| `personal.rs` | GET/POST `/api/schedules`; PUT/DELETE `/api/schedules/{id}`; POST/DELETE `/api/schedules/{id}/share`; GET `/api/shared-schedules/{id}`; GET `/api/profile/courses`; PUT/DELETE `/api/profile/courses/{code}` | Ownership, validation, revisions, persistent saves, history, sharing and revocation |
| `friends.rs` | GET/POST `/api/profile/friends`; POST `/api/profile/friends/{id}/accept`; DELETE `/api/profile/friends/{id}`; POST `/api/profile/friend-invites`; POST `/api/profile/friend-invites/{token}/redeem`; DELETE `/api/profile/friend-invites/{token}`; PUT `/api/profile/friend-visibility` | Profile registration, consent, recipient checks, expiring single-use invitations and visibility |
| `finals.rs` | GET `/api/finals`; POST/DELETE `/api/finals/import` | Private calendar parsing, timezone/term validation, atomic replacement and owner isolation |
| `tests.rs` | Skeleton regression checks | Add feature tests as behavior is built; removed feature tests no longer claim those features work |

## Implementing a handler

1. Read `sites/web/src/lib/contracts.ts` and [frontend contract](api/frontend-contract.md) for request and response shapes. The frontend is the existing integration target.
1. Replace the handler's `Err(not_implemented())` with the smallest complete implementation. Existing underscore-prefixed bindings mark reserved extractor inputs; rename them as you use them.
1. Use `Backend::db` for a read-only catalog connection. Add a focused personal connection accessor around `Backend.users` when needed. Keep SQL parameterized and ownership in every personal-record predicate.
1. Select public versus member data from verified server identity before exposing JSON or database bytes. Reintroduce explicit authorization in each feature; retained extractors with `Result<CurrentUser, StatusCode>` do not enforce it automatically.
1. Preserve wire types or coordinate a deliberate frontend change. Add OpenAPI coverage and tests for success, invalid input, unauthorized access and persistence as relevant.
1. Update the feature status and documentation when the feature actually works.

## Database changes

Catalogs and personal data are separate. `CATALOG_DIR` selects generated catalog artifacts; personal storage uses `USER_DB_PATH`, then `DATABASE_PATH`, then `data/private/users.sqlite`. See [SQLite setup](api/sqlite-backend.md) and [database requirements](api/database-requirements.md).

The retained personal schema has schedules, saved_courses, final_imports, profiles, friendships and friend_invites (`user_version=3`). Startup preserves existing records with `CREATE TABLE IF NOT EXISTS`. Schema changes need versioned migrations and tests opening a previous database. Do not point the catalog importer at personal storage.

## Run and verify

Use the root README setup and `devenv up`. Web API startup still needs the configured OIDC issuer and secrets; legacy API startup does not. An empty development catalog permits startup but fails readiness. Server-backed UI errors are expected until the corresponding handlers are implemented.

From a development shell at the repository root:

```sh
cargo fmt --all -- --check
cargo test -p courses-web-api -p courses-api
cargo check --workspace
python3 -m unittest discover -s scripts -p 'test_*.py'
```

For frontend work, run its existing check/build and browser tests from `sites/web`. Old feature acceptance tests were removed alongside their implementations; the retained backend checks cover identity policy, encrypted catalog integrity and the skeleton response. Add authorization and durability regression tests with each restored feature.

## Suggested implementation order

1. Catalog terms, courses and detail reads, then filter/pagination parity with the frontend.
1. Public/member boundaries and catalog version/download transport.
1. Schedule CRUD, revisions and saved/taking/taken courses.
1. Read-only schedule sharing and revocation.
1. Profiles, friendship consent, invitations and shared-course visibility.
1. Personal finals calendar imports.
1. Full API documentation, feature acceptance tests and deployment smoke checks. Decide separately whether the legacy service needs rebuilding.
