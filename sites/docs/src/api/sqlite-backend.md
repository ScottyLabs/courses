# SQLite backend and browser handoff

> Backend reset: product endpoints currently return 501. This document describes retained infrastructure or the intended integration contract; feature behavior must be rebuilt. See [backend development](../backend-development.md) for current status.

Database foundation retained in `courses-web-api`; intended to replace the web app's DuckDB/custom-binary
catalog transport. The separate `courses-api` crate is not the browser backend.

## Run

```sh
python3 scripts/import_catalog.py data/raw
python3 scripts/verify_catalog.py
# Load the existing OIDC environment using devenv, then:
CATALOG_DIR=data/catalog USER_DB_PATH=data/private/users.sqlite cargo run -p courses-web-api
# In sites/web:
deno task dev
```

`APP_URL` must exactly match the browser origin (including port), e.g.
`http://localhost:5173`. HTTPS enables Secure session cookies. OIDC still uses
the configured Keycloak issuer/client and Ricochet relay. The issuer must supply
`email` and `email_verified: true`; only `cmu.edu` and its subdomains are accepted.
Project membership alone no longer grants access. Verify that the issuer's
university identity mapping actually verifies email ownership before deployment.
No development login bypass exists. Live CMU login was not exercised in this change.

`USER_DB_PATH` (or Kennel's provisioned `DATABASE_PATH`) is a durable SQLite database using WAL and a busy timeout. Mount
it on persistent storage, keep it outside `STATIC_DIR`, and back it up with
SQLite's backup API (do not copy only the main file during writes). This is a
single-host deployment; replicas require a shared transactional storage design.
Sessions currently use the existing in-memory store: restart requires sign-in,
but schedules and saved courses remain. Catalog files are read-only and separate
from personal data. Generated exports and databases are gitignored.

## Target access and endpoints (not implemented)

Anonymous users can read courses, descriptions, prerequisites, units, terms,
gen eds, weekly times, and public catalog requirements. The public DB never
contains structured instructor identity, FCE data, or meeting locations.
CMU users receive a separate member database. JSON uses the same audience split.
`min_fce` queries and instructor routes require CMU identity. Missing login
returns 401; an authenticated non-CMU identity returns 403.

Retained routes (currently 501):

- `GET /api/terms`, `/api/courses`, `/api/courses/{code}`
- `GET /api/instructors`, `/api/instructors/{id}`, `/api/geneds`
- `GET /api/catalog/version`, `/api/catalog/binary`
- `GET /api/programs?search=...`, `/api/programs/{id}`
- `GET, POST /api/schedules`; `PUT, DELETE /api/schedules/{id}`
- `POST, DELETE /api/schedules/{id}/share`; `GET /api/shared-schedules/{id}`
- `GET /api/profile/courses?status=saved`
- `PUT, DELETE /api/profile/courses/{code}` (saved/taking/taken)

Schedule updates require the returned integer `revision`; concurrent stale
updates return 409. Ownership is checked on every mutation. Course and section
IDs must belong to the selected term. Shares are random, read-only, revocable,
and omit owner identity. `program_id` can be stored with a schedule.
All cookie mutations require an exact `Origin: APP_URL`; requests are limited
to 128 KiB. Responses use `private, no-store` and `Vary: Cookie`.

The target behavior is to load and save courses through the API after login. Anonymous stars
remain local drafts, without copying protected fields. Personal mutations never
run against the browser database. `taking` and `taken` are separate from stars
and require `term_id` on mutations; removing a star preserves history.

Friendship requests, expiring invites, explicit sharing consent, persistent course history and personal finals imports need implementation. See [personal API](user.md) for their contracts.
The generated OpenAPI currently describes the auth/health routes; the route list
above and `contracts.ts` describe these new handlers.

## Browser catalog

`public.sqlite` and `member.sqlite` contain the full imported history for server
queries. Smaller `*-browser.sqlite` files contain default course rows plus the
three latest terms; older term queries fall back to the server. Programs remain
API-served. Gzipped browser files are about 4.4-7.0 MB over the wire; sizes vary with catalog coverage. SQLite WASM runs in a worker and is bundled by Vite, with no external CDN.

First reads use JSON while WASM and the correct audience's database load.
After schema validation, covered queries use local SQL. Initialization/query
failure falls back to JSON. The OPFS cache is partitioned by identity/permissions
and content hash, and rechecks server access before opening a cached file.
Downloaded bytes are SHA-256 checked. Account changes terminate the worker;
logout clears the cache. Session checks repeat after 60 seconds of local use.
Revocation cannot retract bytes already delivered to an authorized browser.

Publish the generated directory as a unit (stage a generation, then switch
`CATALOG_DIR` and restart); do not expose raw exports through static hosting or
an unauthenticated object store. The importer retains all source files and
records an inventory; only explicitly selected fields reach the public database.

## Requirements refresh

```sh
python3 scripts/scrape_programs.py
python3 scripts/import_catalog.py data/raw
```

The public scraper discovers links from CMU's Programs A-Z, respects robots.txt,
uses serial requests with a delay and timeouts, and records URL, catalog year,
retrieval timestamp, HTML checksum, requirement text, and referenced courses.
It preserves source HTML and reports failures rather than claiming completeness.
Text is displayed as text, never injected HTML. Course references are not
interpreted as "all required"; electives, grades, double-counting and substitutions
need a separately validated degree-audit evaluator.

Existing Stellic exports retain 2,848 versioned requirement trees. To refresh
them, use `cargo run -p courses-scraper -- --mode programs --concurrency 4` in the
existing authenticated scraping environment. `--mode courses` refreshes course
and section data. A valid Stellic session is required. The Spring 2027 refresh used CourseWeave's public snapshot; no authenticated bulk scrape was run.
Do not use `--incremental` when refreshing stale existing records.

### Targeted Spring 2027 refresh

Sign in to Stellic in a browser. In DevTools -> Network, open
`planner/getstudentprofile/` and copy the **Cookie request-header value** into
`data/private/stellic-cookie.txt` (including `csrftoken`). Do not put the cookie
in a chat, command-line argument, or committed file. The private directory is
ignored by Git. No CMU password, Duo code, Canvas token or LDAP password is needed
for the course/section-only run.

```sh
cargo run -p courses-scraper -- --mode courses --term S27 --concurrency 1 \
  --cookie-file data/private/stellic-cookie.txt --export-root exported/spring-2027
# Use the actual per-user output path reported by the scraper:
python3 scripts/import_catalog.py data/raw --stellic-root exported/spring-2027/ANDREW_ID
python3 scripts/verify_catalog.py
```

Explicit term mode first checks that the public CMU SOC publishes the requested semester, before making authenticated requests. It processes published course codes serially with a two-second delay and stops on the first failed course. This limits traffic but is not a guarantee against account restrictions.

Explicit term mode bypasses the historical relative-year cap and refuses
`--incremental`. It requests course info plus Spring 2027 sections and rejects
returned dated sections for a different term. It records completion/failure
counts in `scrape-manifest.json`; any failed request makes the run unsuccessful.
The refreshed folder overrides matching historical course/term records during
import. Before publishing, inspect actual section counts/dates: a successful empty
response does not establish that Spring 2027 registration data is published.
The existing `physical_year=2026` parameter is retained for the current 2026
Stellic planning context; verify it against a real Spring 2027 browser request
if the service returns a different term or empty results.

## Completion checks

Validated October 23, 2026: 7 backend/auth tests, 4 scraper tests, 10 frontend
tests, 2 CourseWeave conversion tests, Svelte checks (no errors or warnings),
and the production frontend build. Full public/member catalog integrity,
audience isolation, and S27 counts pass. The real browser smoke test passes
API warmup, SQLite WASM worker handoff, public-field exclusion and disposal.

Schedules support reopening, deletion and disabling share links in the UI.
Changing semesters clears selected sections and reloads course details; courses
not offered in the new semester must be removed before saving. Major requirement
references reload with saved schedules.

See the [deployment runbook](../deployment.md) for persistent storage, encrypted catalog releases, the rollout status and live acceptance checks.
