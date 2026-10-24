# 17. Store personal data on the server

Date: 2026-10-24

## Status

Accepted

## Context

Courseweave keeps schedules, degree plans and a student's academic record in the browser's storage and syncs them
to an account in the background. That suits a static site, but it means the browser's copy is what the student
actually works on, and anything not yet synced exists only on that device. The web API already stores schedules,
saved courses and friendships in its SQLite database, scoped to the signed-in owner, with revisions to reject stale
writes.

## Decision

- For a signed-in student, the web API's database is the only durable copy of their schedules, degree plans,
  academic record, saved courses and preferences. The website loads them after sign-in and writes each change back.
  - Each schedule and plan is its own record. The academic record and preferences are one record per student.
  - Every record has a revision. A write names the revision it was based on, and a stale write gets 409 with the
    current record. The website then reapplies its change to the current record, using Courseweave's merge rules,
    and writes again.
  - Writes are debounced on the client and limited in size and rate on the server.
- A signed-out visitor can build a schedule or plan as a draft, kept in `sessionStorage` for that tab only and
  labelled as unsaved. Signing in saves the draft to their account. Nothing personal is written to `localStorage`.
- Device conveniences that aren't personal data, such as the color theme and whether lists show cards or rows, stay
  in `localStorage`.
- Record payloads are typed Rust structs. The server validates them and the OpenAPI document describes them. JSON
  field names are camelCase throughout the API, matching the catalog documents.
- Uploaded SIO and Stellic PDFs are parsed in the browser. Only the parsed courses are sent; the files never leave
  the device.

## Consequences

- A student sees the same data on every device, and losing a browser loses nothing
- Editing needs the API: when it's unreachable, the website says so and keeps the unsaved change on screen to retry
- Personal data in the server's database must be backed up, as the [deployment runbook](../deployment.md) describes
- Renaming the JSON fields changes the existing schedule, saved course and friend endpoints. Nothing has shipped on
  them yet, so they change without a compatibility period
