# Maintainer guide

CMU Courses has two different APIs. The browser uses `courses-web-api`, a Rust/Axum service that serves the built Svelte application, catalog documents, JSON endpoints and authenticated sessions on **one origin**. `courses-api` is the older standalone public catalog service; its custom binary format and publishing workflow are separate. Do not point the new web app at that service.

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
| `crates/courses-index` | Catalog builder: raw inputs to `catalog.sqlite` ([catalog data](api/data.md)) |
| `crates/courses-catalog` | The `catalog.sqlite` format: documents by audience, relational tables, full-text search |
| `sites/web/src/routes/` | Pages: courses, course, requisites, instructors, schedules (with the finals dialog), plan, profile, invite |
| `sites/web/src/lib/features/` | One folder per area; see the [feature pages](features.md) |
| `sites/web/src/lib/components/` | The design system (`ui`, `layout`, `shell`) and shared course, instructor and calendar components ([design system](design.md)) |
| `sites/web/src/lib/core/` | Courseweave's UI-free modules (audits, requisites, plans, search, schedules, imports) and their tests |
| `sites/web/src/lib/api/` | Same-origin API client, errors and the types generated from `openapi.json` |
| `sites/web/src/lib/data/` | Catalog document loading and caching, the session, stale-deploy reloads |
| `sites/web/src/lib/state/` | Server-backed stores for schedules, plans, the record, saved courses and preferences; drafts |
| `sites/web/dev/` | The in-memory mock API for development and tests; the API type generator |
| `scripts/scrape_programs.py` | Public CMU catalog requirement pages (not used by the catalog build yet) |
| `scripts/publish_catalog.py` | Encrypted catalog publishing; never uploads personal data |

## Request lifecycle

The website loads catalog documents from `GET /api/catalog/<name>` and runs Courseweave's code over them in the
browser; personal data is read and written through the JSON routes with revisions. [Website data and
state](frontend.md) describes both, the stores and the development mock.

Authentication gates **bytes**, not just UI elements: the server picks a document's public or member version by the
caller's session, and member data already downloaded by a previously authorized browser cannot be remotely retracted.

## Changing the code

Keep SQL parameterized. Keep ownership in each mutation's SQL predicate, even when the UI has already checked it. Never infer CMU membership from project groups. When adding a protected field, leave it out of the public document versions and the relational tables, and test that (decision 20).

When a catalog document's format changes, update its types in `sites/web/src/lib/core` and the loaders in `sites/web/src/lib/data/catalog.ts` with it. When a personal route changes, regenerate `openapi.json` and the website's types (`deno task generate-api`), and update the mock API in `sites/web/dev`.

Personal schema changes must preserve existing records. Current additions are `CREATE TABLE IF NOT EXISTS`; `PRAGMA user_version` identifies the schema. Future column changes need a version-gated migration, a test opening a previous schema, and a backup/rollback plan. Never run a catalog importer against the personal database.

Requirement text is a reference, not a degree-audit algorithm. Preserve source URLs and catalog/entry years. Do not translate a list of referenced courses into "all required" without interpreting alternatives, minimum grades, double-counting and substitutions.

## Verification

```sh
cargo test -p courses-web-api -p courses-scraper -p courses-index
cd sites/web
deno task check
deno task test
deno task build
```

Use the pinned TypeScript/Svelte checker versions. The previous `@ts-nocheck` on the main page hid an incompatible checker setup; it has been removed. Do not restore it to silence type errors.

Test with two distinct identities for ownership and friendship flows. An anonymous mock cannot validate the university identity provider. Production sign-in must also be tested against the real configured issuer.
