# 39. Build an instructor's FCE rows in the browser from the course documents

Date: 2026-10-24

## Status

Accepted

## Context

The Figma's instructor cards show, for every instructor in the list, an aggregate (workload, teaching rate, course
rate) over the semesters and seasons the student picks, and the FCE rows behind it: semester, course, section,
workload, ratings, respondents and response rate. The catalog's `fce/instructors` document has only each instructor's
totals per course and teaching rating per semester, which can't be filtered by season or limited to recent semesters.
The rows are in `fce/courses/<NN>`, one document per course-number prefix (about 1.6 MB gzipped for all 59). A list
can show 1,600 instructors in a semester and nearly 6,000 across all semesters.

## Decision

- The website builds an instructor's rows itself (`loadInstructorFce` in `lib/features/instructors/fce-aggregate.ts`):
  their `fce/instructors` entry names the courses they have FCEs for, the matching `fce/courses/<NN>` documents are
  loaded, and the rows are kept where the instructor matches by name, the way Courseweave matches FCE names to the
  Schedule of Classes.
- Cards load their rows only when they come near the viewport, and the list renders more cards as it scrolls, so a
  list downloads the prefix documents for the instructors actually looked at. The catalog caches each document for
  the session, so instructors sharing a department share the downloads.
- Aggregates are computed in the browser, weighted by responses, from the rows in the student's window.
- The list's rows (the compact view) show the recent teaching rating from `fce/instructors` alone and load nothing
  more.

## Consequences

- No new catalog document or builder change; the course page's FCE documents serve the instructor pages too
- Scrolling far through the cards view can download most prefix documents, about as much as a student browsing many
  courses would
- Name matching can attribute a row to the wrong person when two instructors in the same course share a last name and
  a compatible first name, as in Courseweave
