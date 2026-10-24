# Personal data and consent

For a signed-in CMU student, the web API's database is the only durable copy of their schedules, degree plans, academic
record, preferences and saved courses ([decision 17](../decisions/0017-store-personal-data-on-the-server.md)). This page
covers who can see and change that data. Routes, shapes and limits are in the [web API reference](web-api.md).

## Who owns what

- Only a signed-in user with a verified `cmu.edu` email (or a subdomain) can read or write personal data. Signed out is
  401; signed in with another account is 403.
- Every record is keyed by the OIDC subject from the session, never by an id the browser sends. Every query that reads,
  changes or deletes a record names its owner in SQL, so another student's id behaves like one that doesn't exist (404).
- Every write needs an `Origin` header equal to the site's origin, so other sites can't make changes with a student's
  cookie.

## Revisions

Each schedule, plan, record and preferences document carries a revision. A write states the revision it was based on;
a stale write is refused with 409 and the current version, which the website merges its change into before writing
again. Nothing is overwritten silently. Writes are limited to 128 KiB a record, 500 writes a record each UTC day and 60
writes a student each minute.

## Imports

- SIO and Stellic PDFs are parsed in the browser; only the parsed courses are sent, as the academic record.
- An SIO calendar (`.ics`) uploaded for final exams is parsed on the server. Only final exams for the term's courses are
  kept; the file and its other events aren't stored. A calendar with no matching finals keeps the previous import.
- Neither is an official transcript or a university-wide exam schedule. The registrar's final exam schedule is a public
  catalog document.

## Sharing

- A share link makes one schedule readable by anyone who has the link. It shows the schedule's term, name and courses,
  never who owns it. Revoking the link or deleting the schedule ends access.
- Friends are mutual: a request (by Andrew ID, for someone who has signed in once) or an invite link must be accepted by
  the other person. Invite links are single use and expire after seven days.
- Friendship shares nothing by itself. Each student separately turns on "share my courses" and "share my schedules";
  both start off, and each applies only to that student's own data. Accepted friends then see:
  - with schedules shared: the student's schedules marked visible to friends, and the courses in them that aren't
    hidden; private schedules are never shown.
  - with courses shared: the courses on the student's record for a term that have no grade yet (in progress). Grades,
    past courses, transfer and exam credit, overrides and the profile are never shown.
- Turning sharing off or removing a friend ends access immediately. Friends see each other's name and Andrew ID.

## Deleting an account

`DELETE /api/account` removes everything the student owns: schedules (and their share links), plans, the record,
preferences, saved courses, final exam imports, invites, friendships, the profile and every session on every device.
It can't be undone.

## Retention and backups

Personal data stays until the student deletes it or their account. Sessions expire after 14 days without use and are
deleted hourly afterwards. The database is backed up by operators as the [deployment runbook](../deployment.md)
describes; backups hold the same personal data and session tokens, so access is restricted to authorized operators.
