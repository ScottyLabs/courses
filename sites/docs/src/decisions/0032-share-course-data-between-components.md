# 32. Share course data between components through a module cache

Date: 2026-10-24

## Status

Accepted

## Context

Course cards, rows and panels appear on the Courses page, the course page, the profile, schedules and instructor
pages. Each shows more than the course record a list hands it: the catalog's requisites and description, FCE hours,
friends taking it and whether the student meets its prerequisites. Courseweave reads these through React hooks over
module-level stores. Passing everything in as props would make every page that lists courses load and thread the same
documents, and `resource()` (frontend.md) loads per component, which for a list of a hundred cards means a hundred
effects asking for the same document.

Svelte 5 has a pitfall here: a reaction doesn't track state it created itself, so state created on first use inside a
`$derived` (the first card to ask) never updates that derived.

## Decision

- `lib/components/course/data.svelte.ts` keeps one load per catalog document (the catalog index, a semester,
  offerings, a course's description, FCE summary and rows, syllabi, friends taking courses in a semester), keyed by
  name and cleared when the session changes (`catalog.generation`). Components call `catalogIndex()`, `courseFce(id)`
  and so on in markup or `$derived` and read `status` and `value`.
- A load keeps plain fields and every read goes through one module-level `$state` counter that each settled load
  bumps, so reads stay tracked whichever reaction created the load. The cost is that any settled load reruns every
  reader; loads settle a handful of times per page.
- `lib/components/course/requisite-context.svelte.ts` builds the student's requisite context for a semester once
  (record, active degree plan, schedules, catalog) with memoized getters instead of `$derived` fields, for the same
  reason.
- Components take simple props (a course or its id, and a semester) and fetch the rest themselves.

## Consequences

- A page shows course components without loading anything for them, and many components share one request
- Course components work anywhere, including pages owned by other features
- Any document settling reruns the course components' derived values; cheap at today's sizes, worth revisiting with
  per-key signals if lists grow much longer than a page
