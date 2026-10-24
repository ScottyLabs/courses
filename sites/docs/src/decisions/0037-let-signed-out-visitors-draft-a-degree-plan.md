# 37. Let signed-out visitors draft a degree plan; degree programs need a CMU sign-in

Date: 2026-10-24

## Status

Accepted. Amends the feature map in [Courseweave features in CMU Courses](../courseweave.md), which listed the plan
page as signed in only.

## Context

Degree programs and their requirement trees come from Stellic, so the `programs/*` catalog documents are member data
([decision 16](0016-serve-the-catalog-as-json-documents.md)) and a signed-out visitor can't audit a plan. The rest of
the plan doesn't need them: semesters, courses, unit and FCE totals (FCEs are member data too and simply don't show),
requisite checks against the catalog, and links to schedules. [Decision 17](0017-store-personal-data-on-the-server.md)
already lets a signed-out visitor build a schedule or plan as a draft in the tab, saved to the account on signing in.

## Decision

- Signed out, `/plan` works as a draft: the visitor can create plans and fill semesters, labeled as kept in this tab
  only, and signing in saves them to the account (decision 17).
- Where degree programs would be, the program picker and the requirements column, the page says that majors, minors
  and requirements come from CMU's degree audit and need a CMU sign-in, with a sign-in button. A signed-in account
  that isn't a CMU one is told the same without the button.

## Consequences

- Visitors can try the planner before signing in, and nothing they plan is lost when they do
- The requirements, the overall progress bar and the requirements count stay empty until the visitor signs in
