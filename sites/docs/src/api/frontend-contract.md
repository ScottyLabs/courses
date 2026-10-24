# Frontend API contract

> Implementation update (2026-10-23): core catalog, auth, schedule, saved-course,
> and requirements routes now exist. See [SQLite backend](./sqlite-backend.md)
> for the current behavior and deferred routes. Older DuckDB notes below are historical.

The Svelte frontend calls same-origin JSON routes defined in
`sites/web/src/lib/contracts.ts`. **This document is a backend implementation
request; these routes have not been implemented here.** The current Rust web API
already owns the OIDC session and serves the frontend, so it should be the
browser-facing origin. No bearer token or Andrew credentials should be stored
in browser storage.

See [database requirements](./database-requirements.md) for durable records,
stable catalog references, ownership rules, indexes, and source-data gaps.
The [browser DuckDB contract](./duckdb-catalog.md) specifies the read-only
database publication format used after the initial API requests.

## Existing authentication routes

| Method | Route | Expected behavior |
| --- | --- | --- |
| `GET` | `/api/me` | `200` with `{ subject, name, can_create, is_admin }`; `401` when signed out. Called at startup. |
| `GET` | `/auth/login` | Starts OIDC and returns to the app. |
| `GET` | `/auth/logout` | Clears the server session and returns to the app. |

The frontend sends the same-origin session cookie with every JSON request.
Backend handlers must authorize each protected field and mutation. Return
`401` for no session and `403` for a signed-in user without permission. Do not
put FCEs, section times/locations, or instructor identity in anonymous JSON.
Use `Cache-Control: private` for authenticated responses. Avoid open redirects
if return-to routing is later added to login.

## Needed data endpoints

| Method | Route | Query | Response / notes |
| --- | --- | --- | --- |
| `GET` | `/api/terms` | — | `Term[]`: `{ id, label, starts_on?, ends_on? }`, newest relevant term first. |
| `GET` | `/api/courses` | `search`, `department`, `level`, `units`, `offered_in`, `min_fce`, `term`, `view`, `page`, `page_size` | `{ items: Course[], total, page, page_size }`. Search code, title, description, and keywords. `offered_in` is `fall`, `spring`, or `summer`; `min_fce` is a numeric overall-course threshold (protected data; reject or ignore for anonymous users). `view` is one of `all`, `fces`, `info`, `sections`; it may be ignored by the backend if one result shape suffices. An empty search returns browsable courses. |
| `GET` | `/api/courses/{code}` | `term` | One `Course` with description, requirements, gen eds, instructors, syllabi, FCE summary, and offered sections allowed for that requester. `404` for unknown code. |
| `GET` | `/api/instructors` | `search` | `{ items: Instructor[], total }`; protect as required by the existing data policy. |
| `GET` | `/api/instructors/{id}` | — | One `Instructor` with courses taught and permitted aggregate FCEs. |
| `GET` | `/api/geneds` | — | `Gened[]`: `{ id, name, description?, courses: string[] }`. |
| `GET` | `/api/finals` | `term`, optional comma-separated `courses` | `FinalExam[]`; dates as ISO 8601 timestamps with timezone. Include course, section, location, start, end. |
| `POST` | `/api/finals/import` | Multipart `calendar` file containing a SIO `.ics` export | `FinalExam[]` parsed for the signed-in user. Validate file type and size, discard unrelated calendar events, and protect the cookie-authenticated mutation against CSRF. The Figma final schedule screen uses this upload flow. |
| `GET` | `/api/schedules` | — | Current user's `Schedule[]`; requires a session. |
| `GET` | `/api/shared-schedules/{shareId}` | — | A shareable read-only schedule, if sharing is enabled. Use an unguessable ID and avoid exposing owner identity by default. |
| `GET` | `/api/profile/courses` | `status=saved|taking|taken`, optional `department`, `level` | `Course[]` for the signed-in user's profile sections. The current UI keeps saved courses locally until this exists. |
| `GET` | `/api/profile/friends` | — | `Friend[]` for the signed-in user: `{ id, name, department?, shared_courses: string[], shared_schedule_id? }`. Return only courses and schedules the friend agreed to share. |

`Course` is `{ code, name, description, units, department, level?,
offered_terms: string[], gened_tags: string[], prerequisites?, corequisites?,
antirequisites?, instructors?, fce?, sections?, syllabi? }`. `fce` contains
`hours_per_week?`, `overall_course?`, `overall_teaching?`, `semesters?`.
`sections` contain `{ id, label, kind, instructors, meetings, capacity?,
enrolled?, waitlist? }`; each meeting has `{ days: string[], starts_at,
ends_at, location? }`. Use `HH:mm` local time for weekly meetings and weekday
tokens `M`, `T`, `W`, `R`, `F`, `S`, `U`. Syllabus links have `{ label, url }`.
`Instructor` is `{ id, name, department?, title?, email?, courses: string[],
fce? }`. `FinalExam` is `{ course_code, course_name, section?, starts_at,
ends_at, location? }`.

## Needed schedule mutations

| Method | Route | Body | Response |
| --- | --- | --- | --- |
| `POST` | `/api/schedules` | `{ name, term_id }` | Created `Schedule`, including server ID. |
| `PUT` | `/api/schedules/{id}` | Entire `Schedule` | Updated `Schedule`; reject a different owner's ID. |
| `DELETE` | `/api/schedules/{id}` | — | `204`; reject a different owner's ID. |
| `POST` | `/api/schedules/{id}/share` | — | `{ share_id }` for a newly created or existing unguessable, read-only share link. Require schedule ownership and allow revocation. |
| `PUT` | `/api/profile/courses/{code}` | `{ status: "saved" }` | Add a saved course to the signed-in profile; idempotent. |
| `DELETE` | `/api/profile/courses/{code}` | — | Remove a saved course; `204`. |
| `POST` | `/api/profile/friends` | `{ andrew_id }` | Add or request a friend relationship; return the permitted `Friend` summary. Validate the Andrew ID and require the other user's acceptance before sharing private data. |
| `POST` | `/api/profile/friend-invites` | — | Return `{ url }` with an unguessable invite token. Expire the link and require authentication and acceptance before creating a relationship. |

`Schedule` is `{ id, name, term_id, items: { course_code, section_ids:
string[] }[], share_id?, updated_at? }`. Validate that section IDs belong to
their course and term. The frontend keeps an anonymous draft in local storage;
it only sends it to the server after sign-in and an explicit save. The API
should enforce request-size limits, input validation, and CSRF protection for
cookie-authenticated mutations (for example, SameSite cookies plus an
Origin/CSRF check). Return a consistent JSON error `{ message }` on failures.

## Lifecycle operations still needed

These complete the storage lifecycle but are not called by the current UI.
Add matching frontend flows when implementing them; the paths below are proposed.

| Method | Route | Behavior |
| --- | --- | --- |
| `PUT` | `/api/profile/courses/{code}` | Extend status support to `taking` and `taken` with a `term_id` and explicit visibility. Saving is independent of course history. |
| `DELETE` | `/api/profile/courses/{code}` | Add status/term selection so removing a star cannot remove course history. |
| `POST` | `/api/profile/friends/{id}/accept` | Accept only an incoming pending request. |
| `DELETE` | `/api/profile/friends/{id}` | Reject/remove a relationship and revoke friend access. |
| `POST` | `/api/profile/friend-invites/{token}/redeem` | Authenticate and explicitly accept a valid invite; consume atomically. |
| `DELETE` | `/api/profile/friend-invites/{id}` | Revoke an invite owned by the current user. |
| `DELETE` | `/api/schedules/{id}/share` | Revoke active share grants; require schedule ownership. |
| `DELETE` | `/api/finals/import` | Remove the current user's import for an explicit term. |

Also extend the wire types for schedule revisions (optimistic concurrency),
private profile-course visibility, explicit finals-import term selection, and
detailed per-term instructor FCE rows. The current summary-only FCE response
cannot populate respondent counts or semester rows accurately. The existing
calendar import endpoint must persist parsed exams for authenticated reloads.

## Current implementation state

- The UI, route navigation, filters, course details, instructor and gen ed
  views, finals export, local schedule draft, weekly selected-section view,
  overlap highlighting, and copyable share links returned by the server are
  implemented in the frontend.
- The OIDC sign-in, sign-out, and identity request use the existing backend
  routes. The frontend does not contain a fake login or credentials.
- Until the JSON endpoints above exist, most views show explicit unavailable
  states. The course catalog shows two visibly labeled local preview examples
  so the Figma card layout can be reviewed. They are replaced by API results
  when the catalog endpoint responds. The frontend lazily loads DuckDB WASM
  and a compatible database file, then uses SQL for covered catalog reads.
  The database publisher remains to be implemented by the backend owner.
- The previous application also had saved schedules, instructors, gen eds,
  finals, search filters, FCEs, course info, and section scheduling. The API
  contract covers their data needs. The frontend can request a share link
  through the documented endpoint and copy a returned `share_id`.
