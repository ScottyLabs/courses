# 44. Put the degree plan's programs and progress in a rail

Date: 2026-10-24

## Status

Accepted. Supersedes [decision 36](0036-lay-out-the-degree-plan-beside-its-requirements.md) on where the plan's header
goes; its semesters beside the requirements, drag and drop and menus stand.

## Context

Decision 36 made `/plan` a full-width page: a header with the plan picker, the programs (each a card with its
requirements version and concentrations), the program search with its type filter, the plan's numbers and the
degree progress bar, then the semesters beside a sticky requirements column. Every other page with tools beside its
content (Courses, Instructors, Schedules, Profile) puts them in the rail: a white column on the left with a title and
Hide that becomes a sheet on phones, and a white band above the content. Next to those pages the plan read as a
different site, and its header pushed the semesters below the fold. The rail was also a fixed 418 pixels on any screen
from 1440 pixels up, leaving wide screens' extra room to the content alone.

## Decision

- `/plan` uses `RailLayout` (title "Plan", remembered Hide under `plan`). The rail holds the programs (name and kind,
  requirements version, concentrations, remove, and the program search, whose type filter shows only in its results),
  the progress (the degree progress bar and the numbers as `StatBox`es in a well, with the first semester), and
  "Requisites", the courses planned before their requisites, each jumping to the course. The band holds the title,
  the plan picker, "New plan" and the plan's options; on phones it also has the button that opens the rail's sheet and
  the progress bar in one line.
- The main column has the record prompt, the semesters and the requirements. The requirements get their own sticky
  column when the main column is at least 900 pixels wide (measured, so hiding the rail can make room), and are a tab
  otherwise. The column is a third of the main column, 320 to 440 pixels (300 to 400 beside the detailed table).
- Requirement rows lay out by the panel's width (a container query): narrower than 32rem, a course's number and its
  Add or status share the first line and its title takes the second; wider, as in the Expand dialog, all sit on one.
- The rail, here and on every rail page, keeps its widths up to 1440 pixels (340, 380, 418) and then grows with the
  window to 560 pixels at 2560 (`RAIL_WIDTH` in `components/layout/rail-context.ts`).

## Consequences

- The plan looks and behaves like the other rail pages: the same title, Hide, sheet and band, and its semesters start
  near the top of the page
- On a 1280-pixel screen with the rail open the requirements are a tab; hiding the rail or a wider screen puts them
  beside the semesters
- Phones open the programs and numbers in a sheet, as Schedules opens its courses; the band keeps the overall progress
  in view
