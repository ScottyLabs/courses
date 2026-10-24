# 36. Lay out the degree plan beside its requirements, not in a rail

Date: 2026-10-24

## Status

Accepted

## Context

The Figma redesign has no degree plan screen, and the design system lists the plan among the pages that use the rail
layout: a filter rail on the left that becomes a sheet below 1024 pixels. The plan page's two halves are the semester
grid and the requirement audit, and students work across them: they add a course the audit suggests, drag it onto a
semester, and watch the requirement fill in. Courseweave puts the semesters on the left and the requirements in a
sticky column on the right that is 400 to 500 pixels wide on wide screens, and switches between them with tabs on
narrower ones. A rail is 340 to 418 pixels, collapsible, and a sheet over the page at tablet widths, which would hide
the semesters while a student picks a course for them.

## Decision

- `/plan` is a full-width page in the design system's visual language (white cards with the Figma border on the
  page background, Inter's type scale, pill buttons and chips), laid out like Courseweave's plan page: a header with
  the plan picker, programs and the plan's numbers, then the semester grid beside a sticky requirements column from
  1180 pixels up, and "Semesters" and "Requirements" tabs below that.
- Drag and drop is on only with a fine pointer at 768 pixels and up, as in Courseweave. Everywhere else, and from the
  keyboard, each course's menu moves it up, down or to another semester, and "Add" in the requirements puts a course
  in the semester the plan suggests and jumps to it.
- The design system's page list no longer names the plan among the rail pages.

## Consequences

- The requirements stay in view while the semesters scroll, and dragging from them into a semester needs no sheet
- The plan page doesn't share the rail's Hide button or phone sheet; its narrow layout is tabs instead
- Course rows, menus and dialogs on the plan come from the design system's primitives, so the page still looks like
  the rest of CMU Courses
