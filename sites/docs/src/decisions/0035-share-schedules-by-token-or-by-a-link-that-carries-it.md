# 35. Share schedules by token when signed in, by a link that carries the schedule when signed out

Date: 2026-10-24

## Status

Accepted

## Context

The web API shares a schedule with a random token (`POST /api/schedules/{id}/share`, read at
`/schedules/shared/<token>`), which only works for a saved schedule and so only for a signed-in student. Signed out,
a schedule is a draft in the tab ([decision 17](0017-store-personal-data-on-the-server.md)). Courseweave shared
schedules with links that encode the schedule itself (`?import=<payload>`, `core/share.ts`), which need no account.

## Decision

- Signed in, sharing creates or revokes the token link, and the shared view offers "Copy to my schedules".
- Signed out, the share dialog offers Courseweave's link with the schedule in it: `/schedules?term=<id>&import=<payload>`.
  Opening one shows the schedule in a dialog (switching to its semester first) and offers to save a copy, signed in or
  out.
- Each schedule's visibility to friends (only me or friends) is set in the same dialog, which says when the student
  doesn't share schedules with friends yet and can turn that on.

## Consequences

- Visitors can pass a schedule along without an account, as in Courseweave; those links can't be turned off and
  carry at most 60 courses
- Two kinds of share links exist, so `ImportLinkDialog.svelte` and the shared route both stay
