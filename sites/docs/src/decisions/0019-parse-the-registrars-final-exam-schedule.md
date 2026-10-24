# 19. Parse the registrar's final exam schedule

Date: 2026-10-24

## Status

Accepted

## Context

CMU Courses checks a student's schedule against final exams ([decision 14](0014-bring-courseweave-features-into-cmu-courses.md)).
The registrar publishes each term's final exam schedule as a PDF printed from a spreadsheet, usually by the sixth
week of classes, with separate PDFs for early minis. There is no other machine-readable source. Students can already
import their own exams from an SIO calendar, but that only covers their own courses.

## Decision

- The PDFs are committed as `data/finals/<term>.pdf`, plus `data/finals/<term>-<part>.pdf` for separately published
  parts such as a mini's exams. They are public documents.
- `courses-index build` extracts their text with the `pdf-extract` crate, marking cell boundaries by the gaps
  between glyphs, and finds each row by its date, time range and delivery mode. It writes `finals/<term>`: a list of
  `{ courseId, section, title, date, start, end, mode, location, instructors }`, sorted, with dates as `YYYY-MM-DD`
  and times as minutes after midnight in Pittsburgh.
- The build fails rather than publish a partial schedule: every time range in the text must belong to a parsed row,
  and every printed weekday must match its date.
- The public version leaves out instructors. Rooms stay, since the registrar publishes them.

## Consequences

- Adding a term's schedule, or an updated one, is committing its PDF and rebuilding
- A change to the registrar's layout fails the build instead of dropping rows silently, and needs a parser change
- The registrar's titles are its own abbreviations; the website can show the catalog's title instead
- Exams published after the PDF, or changed after it, aren't reflected until a new PDF is committed
