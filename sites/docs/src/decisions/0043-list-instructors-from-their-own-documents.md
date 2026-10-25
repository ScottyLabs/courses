# 43. List instructors from their own documents

Date: 2026-10-24

## Status

Accepted

## Context

The instructor list (`/instructors`) built itself in the browser from documents made for other pages: a semester's
whole `term/<id>` (sections, meetings and rooms included) and `fce/instructors` for the rows' ratings, plus
`catalog/index` and `offerings` for course titles. "All semesters" added `teaching` and built the list from all of
them. A QA pass measured about 8.7 MB of decoded JSON before the first card of a semester showed, and about 6.3
seconds to the first card of all semesters in development. Each card then loaded the `fce/courses/<NN>` documents
for its rows ([decision 39](0039-build-instructor-fce-rows-in-the-browser.md)), after looking the instructor up in
`fce/instructors`.

## Decision

- `courses-index` writes member-only `instructors/<termId>` for each semester and `instructors/all`
  (`crates/courses-index/src/instructors.rs`): for each instructor, what the list shows and searches (name,
  departments, courses, section count, course and topic titles, semesters taught), the recent teaching rating and
  the courses `fce/instructors` lists for them. They're built the way the website built the list, with the FCE
  module's port of its name matching, and list instructors in the order the website gives out URL slugs, so links
  don't change.
- The list loads only that document. Cards load their `fce/courses/<NN>` rows from the courses it names, the frame
  after they come near the viewport, without `fce/instructors`.
- The full documents load only while an instructor is open: the semester for its teaching table, `teaching` with the
  FCE index and course lookups for all semesters, and `fce/instructors` for the evaluation history.
- When a catalog was built before these documents (the API answers 404), the website builds the same rows from the
  full documents, as before.

## Consequences

- A semester's list needs about 0.4 MB of JSON (95 KB gzipped) instead of 3.8 MB, and all semesters 1.4 MB (270 KB
  gzipped) instead of 5 MB; with 4x CPU throttling and a fast 4G network, the first card shows about 0.5 seconds
  after the list is requested instead of 1.1, and 0.7 instead of 1.5 for all semesters
- Two more builders of the same list exist, in Rust and in TypeScript (the fallback); a test checks the TypeScript
  side against a document written as `courses-index` writes it, and the slugs agree on every built semester
- The ratings are rounded to 0.01 in the document, so a rating exactly between two hundredths can show 0.01 higher
  than the instructor's page computes
