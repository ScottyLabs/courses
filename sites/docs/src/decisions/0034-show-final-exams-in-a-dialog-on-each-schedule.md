# 34. Show final exams in a dialog on each schedule

Date: 2026-10-24

## Status

Accepted

## Context

The catalog now has the registrar's final exam schedule for a semester (`finals/<id>`,
[decision 19](0019-parse-the-registrars-final-exam-schedule.md)), and students keep their own SIO calendar imports
(`/api/finals`). The Figma has a separate Final Schedule page with a searchable table. What students ask about is
their own exams: when they are, whether two collide, whether three land within a day. Those depend on one schedule's
courses and sections, which the schedules page already has. The registrar lists an exam per lecture (or standalone
section), and some courses give every section the same exam.

## Decision

- Final exams are a dialog on the schedules page, opened from the shown schedule's toolbar, not a page or a navigation
  item. `/schedules?term=<id>&finals=1` opens it, so it can be linked to; `/finals?term=<id>` redirects there.
- The dialog shows the schedule's exams, matched by course and section: the chosen lecture's (or standalone
  section's) exam, else the chosen recitation's, else the course's when every section has the same time, else the
  student's SIO import. A course whose exam depends on a section not chosen yet says to pick one.
- Warnings cover overlapping exams, exams within three hours of each other on the same day, and three or more starting
  within 24 hours (which CMU lets students reschedule). The schedule's exams download as a .ics file.
- The same dialog searches every exam in the semester, says plainly when the registrar hasn't published the schedule,
  and keeps the SIO calendar import for exams the registrar's file lacks.

## Consequences

- Exams are always seen in the context of a schedule, with its warnings; there is no page for browsing all exams
  without one, only the search inside the dialog
- The matching is in `features/finals/finals.ts` and tested there; when the registrar changes how it names sections,
  that's the place to change
- Old links to `/finals` keep working through the redirect
