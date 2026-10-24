# 42. Ask before adding signed-out drafts to an account

Date: 2026-10-24

## Status

Accepted. Amends [decision 17](0017-store-personal-data-on-the-server.md) and
[decision 37](0037-let-signed-out-visitors-draft-a-degree-plan.md), which save a tab's signed-out drafts to the account
that signs in.

## Context

Signed out, schedules, degree plans, the academic record and saved courses are drafts in the tab, and signing in
saved them into the account without a word. On a shared or lab computer the next person to sign in in that tab, or a
student who signed in to the wrong account, got someone else's schedule, plan or courses in their account. Drafts
also used to keep the same names as the account's own ("Schedule 1" twice).

## Decision

- On signing in, the tab's drafted schedules (those with courses; an empty default isn't asked about), plans, record
  and saved courses are held back (`cmucourses:held:<store>` in sessionStorage) while the account loads. Preferences
  and edits made after signing in are saved as before.
- A dialog says what the tab made and asks: "Add to my account" adds it (schedules and plans under names the account
  doesn't use, the record merged with Courseweave's rules, saved courses added); "Discard" forgets it. It can't be
  dismissed without a choice, and a reload asks again.
- Signing out instead gives the tab its drafts back, so a student who signed in to the wrong account can sign in to
  the right one and add them there.
- The record onboarding waits until the question is answered.

## Consequences

- Nothing reaches an account without the student saying so; one more dialog for students who sign in after trying
  the site signed out
- `CollectionStore`, `DocumentStore` and `SavedStore` gain `holdDraft`, `adoptHeld` and `dropHeld`;
  `personal.heldDrafts`, `acceptDrafts()` and `discardDrafts()` drive the dialog (`HeldDraftsDialog.svelte`)
