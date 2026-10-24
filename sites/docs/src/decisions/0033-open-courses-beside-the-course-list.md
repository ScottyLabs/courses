# 33. Open courses in a panel beside the course list

Date: 2026-10-24

## Status

Accepted

## Context

The Figma shows the Courses page as a filter rail and a column of course cards that expand in place, and has no
course page. Courseweave lists courses as compact rows, opens one in a detail panel beside the list, and offers a full
course page from there. Students compare courses while keeping their search and filters; leaving the list for every
course loses that, and expanding cards in place can't hold sections, requisite trees and FCE history.

## Decision

- The Courses page keeps the Figma's rail, toolbar and cards, with Courseweave's compact rows as the other view.
- Choosing a course opens it in a detail panel (`?course=15-122` on the Courses page): beside the results from 1440px
  wide, where the Figma rail leaves room for both, and in place of the results below that, with a link back. The panel
  is the course page's sections stacked in one column; its Expand link opens the full page `/course/15-122?term=`.
- The panel's state is in the URL like the rest of the page's query, so opening a course is a history entry, Back
  closes it, and the link can be shared.
- Cards keep their in-place expansion for the description and FCEs.

## Consequences

- The course detail exists twice, as a panel and a page, built from the same section components
- Between 1024px and 1440px a course replaces the list instead of sitting beside it
- Course links inside the panel open in the panel (through `setCourseHref()`), so following a prerequisite keeps the
  list's state
