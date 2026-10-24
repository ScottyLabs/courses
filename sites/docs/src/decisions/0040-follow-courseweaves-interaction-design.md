# 40. Adapt the Figma design to the features, borrowing from Courseweave

Date: 2026-10-24

## Status

Accepted. Amends [decision 14](0014-bring-courseweave-features-into-cmu-courses.md), which made the
[Figma redesign](https://www.figma.com/design/vsvdhGC2QP5fsvG0Pl7W6v/CMUCourses-Redesign--summer-25---Copy-?node-id=502-10393)
the layout to follow.

## Context

The Figma redesign is from summer 2025 and predates most of the features CMU Courses is gaining: requisite checks,
the degree plan, the academic record, the schedule generator and the all-semesters views. Its screens were drawn for a
smaller product. Courseweave's screens were built around those features, and the project owner likes many of its
choices.

## Decision

- The Figma design stays the base: its look and its layout language (the header and navigation, the left rail,
  course cards, toolbars, the profile layout).
- Screens don't have to match the Figma one to one. Where a Courseweave pattern serves a feature better, such as a
  detail panel beside the course list on wide screens, schedule tabs, or denser course page sections, we use it,
  built from the CMU Courses design system.
- Screens the Figma doesn't have, such as the degree plan and the record, follow Courseweave's structure in the same
  design system.

## Consequences

- Screens can differ from the Figma file, which is a starting point rather than a specification
- Every screen still uses the same components and tokens, so the product looks like one design
