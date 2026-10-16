# Database requirements

This is the persistence specification for the redesigned frontend and its
[API contract](./frontend-contract.md). It describes required data and
invariants; it does not introduce a database, migrations, or backend handlers.
The browser catalog uses DuckDB WASM; its exact publication schema is in the
[browser DuckDB contract](./duckdb-catalog.md). The server's durable application
database engine remains your choice. Table names below describe
a logical relational model; another storage model must preserve the same
ownership checks, uniqueness, and atomic updates.

## Existing catalog and new application data

The scraper and `courses-index` already produce a versioned `catalog.bin`
published to S3. Keep that pipeline as the source for catalog data. The JSON
API can read the index directly or maintain a query projection in a database;
duplicating the entire catalog in the application database is not required.
Schedules, profile course lists, friendships, sharing grants, and imported
finals need durable application storage independent of catalog rebuilds.

Use UTC timestamps for record creation, modification, expiry, and absolute
exam times. Store weekly meeting times as local clock times together with an
IANA campus timezone, normally `America/New_York` for Pittsburgh. Keep unknown
values nullable; missing ratings, enrollment, and meeting times are not zero.

### Stable identifiers

- Give application records opaque stable IDs. Resolve the authenticated
  principal to an internal user ID using the unique pair `(oidc_issuer,
  oidc_subject)`; names and Andrew IDs are not identity keys.
- Use canonical course codes such as `15-251` for durable course references.
  The dense `CourseId` in `crates/courses-index/src/doc.rs` is assigned at
  build time and must not be persisted as a cross-build foreign key.
- Terms need stable IDs with unique `(year, semester)` values. Preserve the
  existing Fall/Spring/Summer mapping and translate it explicitly to API IDs.
  A label such as `Fall 2026` is display text, not the primary key.
- Sections need stable IDs scoped to their course, term, and campus. Keep the
  source's section ID and label separately. Instructor API IDs must also
  survive catalog rebuilds; retain source identity mappings rather than
  matching only on display names.
- Catalog refreshes must not delete historical user records. Keep retired
  course/section references or resolve them through a stable reference table;
  flag unavailable selections instead of silently changing a saved schedule.

## Durable application records

All records below require `created_at`; mutable records also require
`updated_at`. Foreign keys to users must use internal user IDs.

| Record | Required fields | Relationships and constraints |
| --- | --- | --- |
| `users` | `id`, `oidc_issuer`, `oidc_subject`, `display_name`, nullable verified `andrew_id`, optional department, `last_seen_at` | Unique issuer/subject pair. Normalize verified Andrew IDs for exact lookup and enforce uniqueness when present. Populate the Andrew ID only from a trusted identity claim or verified directory mapping; the existing `/api/me` response does not supply it. |
| `profile_courses` | `user_id`, `course_code`, `status`, optional `term_id`, `visibility` | Status is `saved`, `taking`, or `taken`. Saved is independent of enrollment/history: starring a course must not erase a taking/taken record. Saved membership is unique per user/course; taking/taken membership is unique per user/course/term/status. Require a term for taking/taken. Visibility defaults to private and explicitly controls which entries friends may see. |
| `schedules` | `id`, `owner_user_id`, `name`, `term_id`, `revision` | Every schedule has one owner and one term. Revision increments on update so concurrent edits can be rejected rather than overwritten silently. |
| `schedule_items` | `id`, `schedule_id`, `course_code`, `position` | Unique schedule/course pair; a course may be added before selecting sections. Position preserves display order. |
| `schedule_item_sections` | `schedule_item_id`, `section_id` | Unique item/section pair. Every selected section must belong to the item's course and schedule's term. Preserve multiple section selections, such as lecture plus recitation. |
| `schedule_shares` | `id`, `schedule_id`, `token_hash`, `created_by_user_id`, optional `expires_at`, nullable `revoked_at` | Unique cryptographically random token hash. A grant is read-only and may be revoked by the schedule owner. The opaque `share_id` in URLs is the presented token, not the schedule's primary key. Never expose owner identity or bypass CMU authentication for protected catalog fields. |
| `friendships` | `id`, canonical `user_low_id`, `user_high_id`, `requested_by_user_id`, `status`, nullable `accepted_at` | Unique unordered user pair; no self-friendships. Status is `pending` or `accepted`; only the recipient may accept a request. A request alone grants no access. Removing a relationship revokes friend access immediately. |
| `friend_invites` | `id`, `owner_user_id`, `token_hash`, `expires_at`, nullable `redeemed_by_user_id`, nullable `redeemed_at`, nullable `revoked_at` | Unique random token hash, single use, finite lifetime. Redeeming requires an authenticated different user and explicit acceptance. Repeated or concurrent redemption must not create duplicate relationships. |
| `friend_schedule_grants` | `owner_user_id`, `friend_user_id`, `schedule_id` | Unique owner/friend/schedule grant. Require an accepted friendship and schedule ownership. The API's `shared_schedule_id` is a permitted read-only share token, never an owner's private schedule ID. |
| `final_imports` | `id`, `owner_user_id`, optional `term_id`, `imported_at`, `content_hash` | Records a successful SIO calendar import. Associate rows only with the authenticated importer. Persist parsed exam data, not the uploaded calendar or unrelated personal events. |
| `imported_final_exams` | `id`, `import_id`, source event UID, optional recurrence ID, `course_code`, `course_name`, optional section/location, `starts_at`, `ends_at` | Unique source event UID/recurrence identity within an import; end must follow start. Normalize absolute times while preserving the source timezone when needed. Do not treat a missing section as a fabricated section match. |

An invite token may be returned once when created and stored only as a hash.
For schedule shares, the current UI can receive a fresh token each time
`POST /api/schedules/{id}/share` is called, so the backend need not recover a
previous plaintext token from its hash. Define token expiry in configuration
and enforce expiry/revocation on every read.

`GET /api/finals` must include the user's active imported finals after a page
reload. Replace the user's previous import for the same term atomically only
after the new file parses and validates successfully. Re-uploading identical
content is idempotent. Reject an ambiguous term or request an explicit term
rather than assigning exams to the current term by guesswork. If an official
finals feed is added, store its provenance separately and let the user's
import override a matching official exam without exposing private rows.

## Catalog data the API must be able to query

These records may remain in the published index or be stored as a database
projection. Store a source/version reference and refresh timestamp so that
responses and derived aggregates can be traced to a catalog version.

| Data | Required contents and query relationships |
| --- | --- |
| Terms and campuses | Stable ID, year, semester, label, optional start/end dates; campus ID/name/timezone. Sort by actual term chronology. |
| Courses | Canonical code, title, description, department, school, level, units and available unit range, attributes/keywords, website, requirement text, corequisite/antirequisite/equivalent course references. Preserve decimal units. |
| Offerings | Course, term, campus; distinguish actual historical offerings from predicted availability. Support offered-in filtering without parsing a display label. |
| Gen eds | Stable tag ID, name, description if available, course associations, and applicable semester/catalog/program scope when supplied by the source. Do not collapse source-specific scopes into an unconditional eligibility claim. |
| Sections and meetings | Course, term, campus, source section ID, display label, kind, date range where available, nullable capacity/enrollment/waitlist, instructors; **all** meeting blocks with weekdays, local start/end, timezone, building/room. Represent asynchronous/TBA meetings explicitly rather than as a midnight class. |
| Instructors | Stable ID, source identities, name, department(s), nullable title/email, course/term/section teaching relationships. Names are not unique. Serve only fields permitted by the access policy. |
| FCE observations | Source row identity, course, term, section, nullable instructor identity and source matching key, nullable workload and rating fields, respondent count, response rate, enrollment, and provenance. Keep raw observations to support term/instructor filters and aggregate tables. Missing scores must remain null and be excluded from score averages. |
| FCE aggregates | Course or instructor key, selected term range, included semester count, workload/ratings, aggregation method/version. Recompute from observations on refresh; define the weighting and denominator. Keep respondent counts and response-rate units explicit. |
| Syllabi | Course, term, optional section, label, object key or validated URL, format, source/version. Keep files in object storage; database records hold references and access metadata. |

Search needs normalized course code/title/description/keywords and instructor
name fields. Course filters need department, level, numeric units, offering
season/term, and permitted FCE aggregate values. Return a stable ordering with
a deterministic tie-breaker so pagination does not duplicate or skip rows.

### Gaps in the current catalog projection

- `load/sections.rs` reads `raw.timings.first()`. It must retain every timing
  for accurate weekly schedules and conflict detection. The current
  `SectionTime` shape also lacks the API's display section label and optional
  enrollment fields; extend the projection or source these fields elsewhere.
- `doc.rs` already retains detailed `FceRow` observations, while the frontend
  contract currently exposes only summaries. The Figma instructor table's
  semester/section rows, respondents, and response rates require an additional
  response field or endpoint before those cells can be populated accurately.
- Prerequisite text exists, but personal prerequisite satisfaction requires
  the user's course history plus parsed requirement logic. Preserve the
  original expression and AND/OR structure if evaluated. Do not infer
  satisfaction from a saved course, schedule draft, or preview fixture.
  Transcript import and a requirement evaluator are not implemented.

## Ownership, privacy, and authentication storage

Derive ownership from the server session. Never accept a client-supplied owner
ID as authorization. Friendship visibility and share grants must be checked
when responding, including for cached results. Public catalog projections
must exclude FCEs, instructor identity, and section times/locations according
to the [access policy](./usage.md). A share token does not override that policy.
An anonymous share response may include schedule name and course selections;
protected meeting details still require CMU authentication.

Roles such as `can_create` and `is_admin` currently come from OIDC group
claims. Do not copy browser flags into authoritative database permissions.
Store no Andrew passwords. If server-side session persistence is added, use
the session library's schema with session ID, protected payload, and expiry,
and delete expired/logged-out sessions. The current web API uses an in-memory
`MemoryStore`; production with multiple instances or restart persistence needs
a shared session store. Keep OIDC tokens, if required by that library, confined
to protected server-side session storage and out of application logs.

## Required indexes and atomic operations

In addition to primary keys and the uniqueness rules above, provide indexes
for these access paths (or equivalent indexes in the chosen storage system):

| Query | Index/key access path |
| --- | --- |
| Identity lookup and Andrew ID friend lookup | Unique issuer/subject and normalized verified Andrew ID |
| Profile lists | User/status/term and user/visibility |
| Schedule list and children | Owner/updated time, schedule item order, item sections |
| Share and invite lookup/cleanup | Unique token hash, expiry, schedule ID, invite owner |
| Friend lists and pending requests | Both participant IDs/status, request recipient/status |
| Imported finals | Import owner/term, exam import/start time, source event identity |
| Catalog lookup and filters | Unique course code, course/term/campus offering, section course/term, gen ed/course associations |
| Search and FCE queries | Full-text/search index as needed; course/term and instructor/term observation keys; aggregate filter keys |

Replace a schedule and its selected sections in one transaction. Validate all
references before committing, and reject a stale revision with `409` once
revision checking is exposed in the API. Names and IDs need length limits;
arrays, saved schedules, and uploads need configured limits. A scheduling
conflict is a warning, not a database uniqueness violation.

Delete a schedule's items, selections, and grants together. Removing a
friendship must remove its explicit grants. Account deletion must remove
private profile data, schedules, grants, invites, and imported finals while
leaving shared catalog records intact. Use versioned migrations and durable
backups for application data; rebuilding `catalog.bin` cannot restore it.

## Backend and frontend follow-up

- Connect the frontend's local saved courses to profile persistence after
  login using an explicit, idempotent merge. Currently stars remain in
  `cmucourses-saved`; anonymous schedule drafts remain in
  `cmucourses-draft` until explicitly saved.
- Add lifecycle API operations for friendship acceptance/removal, invite
  redemption/revocation, schedule-share revocation, imported-final deletion,
  and taking/taken updates. The current frontend does not call those yet;
  the contract lists them as follow-up requirements.
- Extend the wire contract for schedule revisions, detailed FCE rows, and
  explicit finals-import term selection. Deploy matching frontend changes
  with those extensions rather than requiring fields the current client
  does not send.
- Choose the storage engine, token lifetimes, record/upload limits, backup
  policy, and account-data retention policy before production rollout.
  These are configuration and deployment decisions, not implemented defaults.
