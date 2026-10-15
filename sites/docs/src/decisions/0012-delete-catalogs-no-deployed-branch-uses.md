# 12. Delete catalogs no deployed branch uses

Date: 2026-10-15

## Status

Accepted

## Context

Catalogs for old schemas accumulate in object storage. `main`, `staging`, `dev`, and each open pull request's preview can each use a different one, so we can't keep only `main`'s.

Raw scrape output accumulates too, but we keep all of it as a historical record, since a past scrape can't be redone.

## Decision

A cleanup job runs on a schedule and whenever a pull request closes. It:

- Lists the deployed branches: `main`, `staging`, `dev`, and the head of every open pull request
- Computes each branch's schema hash from the schema SQL at its head, using `git show`, without building
- Deletes every `catalog-<hash>.sqlite` whose hash isn't in that set

## Consequences

- The schema hash has to stay computable from the schema SQL alone
- The job needs permission to delete from the bucket and a Forgejo token to list pull requests
