# 40. Follow Courseweave's interaction design where it differs from the Figma

Date: 2026-10-24

## Status

Accepted. Amends [decision 14](0014-bring-courseweave-features-into-cmu-courses.md), which made the
[Figma redesign](https://www.figma.com/design/vsvdhGC2QP5fsvG0Pl7W6v/CMUCourses-Redesign--summer-25---Copy-?node-id=502-10393)
the layout to follow.

## Context

The Figma redesign is from summer 2025 and predates most of the features CMU Courses is gaining: requisite checks,
the degree plan, the academic record, the schedule generator and the all-semesters views. Its screens were drawn for a
smaller product. Courseweave's screens were built and refined around those features, and the project owner prefers
many of its choices.

## Decision

- Courseweave's UI is the reference for layout and interaction: the explorer's compact list with a detail panel
  beside it, the course page, the filter panel, the schedule planner, the degree plan, the record and the instructors
  page.
- The design system keeps the CMU Courses identity from the Figma: colors, type, the header and navigation, buttons,
  chips and the course card.
- The Figma's course cards stay as one of the two views every course and instructor list offers. The other is
  Courseweave's compact rows.

## Consequences

- Screens can differ from the Figma file, which is no longer the specification
- New features follow Courseweave's patterns, so the product stays consistent as it grows
