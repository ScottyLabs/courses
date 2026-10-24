# 28. Replay edits onto the stored record after a conflict, and keep unsaved changes in the tab

Date: 2026-10-24

## Status

Accepted

## Context

[Decision 17](0017-store-personal-data-on-the-server.md) keeps schedules, plans, the academic record, saved courses
and preferences on the server, each with a revision, and has the website reapply a change when its write is stale
(409), "using Courseweave's merge rules". Those rules (`core/sync-merge.ts`) combine two whole copies: newer
`updatedAt` wins for a schedule or a plan, and the record is a union. They were written for documents synced in the
background between devices that each held everything; applied to a schedule, the newer copy wins outright, so an edit
made on one device can erase another device's edit made a moment earlier. The website also needs somewhere to keep a
change the server hasn't accepted yet: the student may lose the connection, reload, or leave for the sign-in page.

## Decision

- Every record is saved by its own `RecordSync` (`src/lib/state/sync.ts`). It applies edits at once, writes after a
  1.5 second pause, and remembers the edits made since the revision the server last confirmed. On a 409 it replays
  those edits onto the stored record from the response and writes again, up to three times in a row. Edits are the
  store actions' pure functions (`core/state`), so replaying one means "add this course" or "rename to this", not
  "write this copy".
- When the edits aren't known, because the change came from a draft or from storage after a reload, Courseweave's merge
  rules combine the two copies, as decision 17 says.
- Signed out, each store's whole state is a draft in sessionStorage. Signed in, whatever the server doesn't have yet is
  kept in sessionStorage too, with the account it belongs to and the revision it was based on. Loading the account
  folds both in: drafts are created or merged, pending changes are written against their old revision (and merged if
  that's stale).
- A session that ends on its own (a 401) stops writing but keeps the changes on screen and in storage, so signing in
  again saves them. Signing out on purpose saves first, or asks, and then forgets the tab's copy.
- Saved courses have no revisions: each change is a `PUT` or `DELETE` of one course, sent in order, and the last change
  wins.

## Consequences

- Two devices editing the same schedule both keep their changes unless they change the same field, where the later
  write wins
- A schedule or plan deleted on one device while edited on another comes back, as in Courseweave: nothing on a device is
  dropped because the server lacks it
- sessionStorage holds personal data the server hasn't accepted yet, for as long as the tab is open; it never goes to
  localStorage, and signing out clears it
- Server writes are at most one per record per pause, well within the API's limit of 60 writes a minute
