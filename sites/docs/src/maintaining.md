# Maintainer guide

CMU Courses has two different APIs. The browser uses `courses-web-api`, a Rust/Axum service that serves the built Svelte application, JSON endpoints, authenticated sessions and SQLite downloads on **one origin**. `courses-api` is the older standalone public catalog service; its custom binary format and publishing workflow are separate. Do not point the new web app at that service.

## Read this first

- [Catalog data and refreshes](api/data.md): source precedence, schemas, privacy and release artifacts.
- [Web API](api/web-api.md) and [personal data and consent](api/user.md): routes, sign-in, records, ownership, revisions, consent and imports.
- [Deployment runbook](deployment.md): storage, configuration, release and rollback.
- [Feature status](features.md): implemented behavior and explicit limitations.
- [Data audit](api/data-audit.md): dates, record counts and source completeness.

## Code map

| Location | Responsibility |
| --- | --- |
| `crates/courses-web-api/src/main.rs` | Configuration, sign-in mode, startup and the `openapi` command |
| `app.rs` | Router, OpenAPI document, Origin guard, security headers, session and OIDC layers, static site |
| `auth.rs`, `auth/dev.rs` | Verified CMU email policy, `CurrentUser`/`Viewer`/`Writer` extractors, `/api/me`, login and logout; development sign-in (`dev-auth`) |
| `sessions.rs` | `tower-sessions` store over the personal database, with hashed ids and owner tags |
| `db.rs` | Personal database connection, migrations to `user_version` 4 and the v3 conversion |
| `error.rs` | `{ code, message }` errors and conversion of axum's plain-text errors |
| `ratelimit.rs`, `state.rs` | Per-user write limit; shared application state |
| `catalog/mod.rs` | `catalog.sqlite` reader pool and catalog documents by audience, with ETags |
| `catalog/courses.rs` | Public course API over the relational tables |
| `catalog_release.rs` | Download, authenticate and atomically install the pinned `catalog.sqlite` release |
| `records/mod.rs` | Shared validation, revisions and daily write caps |
| `records/schedules.rs`, `records/plans.rs` | Schedules (checked against the catalog) and share links; degree plans |
| `records/documents.rs`, `records/saved.rs` | The academic record and preferences; saved courses |
| `friends.rs` | Profiles, requests, invites, sharing consent, friends' schedules and courses |
| `finals.rs` | SIO calendar parsing and personal final exam imports |
| `syllabi.rs` | Member-only syllabus files from object storage or a directory |
| `seo.rs` | Course page head tags, sitemap and robots.txt |
| `account.rs` | Account deletion |
| `tests.rs` | Router-level tests: ownership, revisions, validation, audiences, syllabi, SEO, friends, deletion, limits |
| `sites/web/src/lib/contracts.ts` | Browser wire types and same-origin HTTP client |
| `data-client.ts` | API-first reads, worker handoff, session rechecks and fallback |
| `catalog.ts` | Download validation and identity-scoped OPFS cache |
| `local-catalog.ts`, `sqlite.worker.ts`, `sqlite-catalog.ts` | Worker lifecycle, SQLite runtime and parameterized queries |
| `components/CourseHistory.svelte`, `components/Friends.svelte` | Personal-history and consent UI |
| `routes/+page.svelte` | Main navigation, catalog, schedules and requirement reference UI |
| `scripts/import_catalog.py` | Local source normalization and independently sanitized database generation |
| `scripts/courseweave.py` | Explicit S27 snapshot conversion and source checks |
| `scripts/scrape_programs.py` | Public CMU catalog requirement discovery and retrieval |
| `scripts/publish_catalog.py` | Encrypted catalog publishing; never uploads personal data |

## Request lifecycle

1. The page requests identity and terms. An anonymous identity result starts a public database warmup; a verified CMU identity starts a member warmup.
1. Search uses JSON immediately. Download and WASM initialization happen independently in a worker.
1. After checksum and schema validation, queries covered by the browser database run locally. Uncovered historical terms and programs continue through the API.
1. Failure to initialize or query locally returns the client to the API path. Personal writes always use the server.
1. An account change disposes the old worker. Logout clears the browser catalog cache. After 60 seconds of local use, identity is checked again before another local read.

Authentication gates **bytes**, not just UI elements. Public SQLite is generated independently and does not contain the member fields even in deleted/free pages. Member data already downloaded by a previously authorized browser cannot be remotely retracted.

## Changing the code

Keep SQL parameterized. Keep ownership in each mutation's SQL predicate, even when the UI has already checked it. Never infer CMU membership from project groups. When adding a protected field, update both the JSON/public export policy and `verify_catalog.py`.

When changing catalog queries, update both `backend/catalog.rs` and `sqlite-catalog.ts`. Add coverage metadata only when the browser export actually includes that dataset/term. The fallback depends on honest coverage declarations.

Personal schema changes must preserve existing records. Current additions are `CREATE TABLE IF NOT EXISTS`; `PRAGMA user_version` identifies the schema. Future column changes need a version-gated migration, a test opening a previous schema, and a backup/rollback plan. Never run a catalog importer against the personal database.

Requirement text is a reference, not a degree-audit algorithm. Preserve source URLs and catalog/entry years. Do not translate a list of referenced courses into "all required" without interpreting alternatives, minimum grades, double-counting and substitutions.

## Verification

```sh
cargo test -p courses-web-api -p courses-scraper
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/verify_catalog.py
cd sites/web
deno task check
deno task test
deno task build
deno task test:browser
```

Open `http://127.0.0.1:5176/tests/sqlite-browser.html` for the real WASM worker smoke test. Its fixture server exposes only the **public** database. It must report API warmup, SQLite handoff, field exclusion and disposal as PASS.

Use the pinned TypeScript/Svelte checker versions. The previous `@ts-nocheck` on the main page hid an incompatible checker setup; it has been removed. Do not restore it to silence type errors.

Test with two distinct identities for ownership and friendship flows. An anonymous mock cannot validate the university identity provider. Production sign-in must also be tested against the real configured issuer.
