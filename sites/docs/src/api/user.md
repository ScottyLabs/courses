# Personal data and consent

> Backend reset: product endpoints currently return 501. This document describes retained infrastructure or the intended integration contract; feature behavior must be rebuilt. See [backend development](../backend-development.md) for current status.

All endpoints below require a verified CMU identity. Every mutation requires an exact `Origin` matching `APP_URL`. JSON errors use `{ "message": "..." }`; 401 means no session, 403 means disallowed identity/origin, 404 means no accessible record, and 409 means a stale schedule revision. The request body limit is 128 KiB.

## Storage and authentication

`USER_DB_PATH` overrides `DATABASE_PATH`; otherwise local development uses `data/private/users.sqlite`. Kennel provisions `DATABASE_PATH` when `scottylabs.sqlite.enable` is set. SQLite uses WAL, a five-second busy timeout and a process mutex for personal writes. The current server is intended for one service instance per personal database.

Ownership keys are the OIDC subject, never a browser-supplied user ID. Identity is established by the configured issuer, token validation and `email_verified=true` with domain `cmu.edu` or a CMU subdomain. Project groups do not grant membership. The profile directory records the verified email local part as the Andrew ID on sign-in; aliases therefore follow the issuer's mapping. Users must sign in once before Andrew-ID requests can find them.

Sessions are currently in memory and expire after one hour of inactivity. Restart requires signing in again but does not remove saved data. OIDC callback state is handled by the existing relay flow. Production cookies are Secure and SameSite=Lax.

## Schedules

| Endpoint | Behavior |
| --- | --- |
| `GET /api/schedules` | List the caller's schedules |
| `POST /api/schedules` | Create `{name, term_id}` with empty items and revision 1 |
| `PUT /api/schedules/{id}` | Replace name, term, items and optional program_id with matching revision |
| `DELETE /api/schedules/{id}` | Delete an owned schedule |
| `POST /api/schedules/{id}/share` | Create/reuse an unguessable read-only share token |
| `DELETE /api/schedules/{id}/share` | Revoke the token |
| `GET /api/shared-schedules/{token}` | Public read-only name, term and course/section IDs; no owner or restricted course fields |

Items are `{course_code, section_ids}`. Each course and section must belong to the term. Names are 1-120 characters; schedules allow at most 50 courses and 20 section IDs per course. Stale saves return 409; reload the saved version before editing again. Anonymous drafts remain local until explicit save after login.

## Saved courses and history

`GET /api/profile/courses?status=saved|taking|taken` lists one category. History rows include `term_id`. `PUT /api/profile/courses/{code}` takes `{status, term_id?}`; `DELETE` takes those fields as query parameters. History requires a term in which the course exists. Saved stars are independent of history, so removing a star never deletes a taken-course record.

The profile UI allows adding and removing course history. These records are not imported grades or an official transcript.

## Friends

| Endpoint | Behavior |
| --- | --- |
| `GET /api/profile/friends` | Accepted friends plus incoming/outgoing requests |
| `POST /api/profile/friends` | `{andrew_id}` creates a pending request |
| `POST /api/profile/friends/{id}/accept` | Only the recipient can accept |
| `DELETE /api/profile/friends/{id}` | Either participant can decline/cancel/remove |
| `POST /api/profile/friend-invites` | Create a seven-day, single-use link; replaces the owner's previous link |
| `POST /api/profile/friend-invites/{token}/redeem` | Explicit signed-in acceptance, consumed transactionally |
| `DELETE /api/profile/friend-invites/{token}` | Owner revokes an invite |
| `PUT /api/profile/friend-visibility` | `{share_courses: boolean}` changes explicit history-sharing consent |

Friend acceptance alone discloses no history. Shared courses are the intersection of both users' taking/taken records in the same term and are returned only when **both** users opted in. Removing a friendship immediately removes that access. Schedules require separately created share links; friendship does not grant schedule access.

## Finals import

`POST /api/finals/import?term=S27` accepts one multipart field named `calendar`, a UTF-8 `.ics` file up to 120 KiB. Imports are private and keyed by owner and term. `GET /api/finals?term=S27&courses=15-122,21-127` supports an optional course filter. `DELETE /api/finals/import?term=S27` removes only the caller's import for that term.

The parser keeps final-exam events with recognizable course codes in the selected term's catalog. Unrelated events are discarded. UTC and IANA `TZID` timestamps are supported; floating times default to America/New_York. Ambiguous local times, invalid durations, recurring final events and dates in a different term are rejected. A failed or empty import preserves the previous import. Original calendars and unrelated events are not retained.

The UI exports stored finals as an iCalendar file. The app does not claim an authoritative university-wide finals feed or infer finals from weekly class times.
