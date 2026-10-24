# Course explorer

## Course page

`/course/[code]?term=` is Courseweave's full course page (`CourseFullPage.tsx`), built from section components in
`sites/web/src/lib/features/course/` and shared renderers in `sites/web/src/lib/components/course/`. The route is a
thin wrapper around `CoursePage.svelte`.

Which semester it shows:

- `term=<id>`: that semester's sections. A semester that doesn't offer the course shows `NotOfferedHere`, with links
  to the semesters that do and to the requisite explorer.
- `term=all`: the all-semesters view (Courseweave's `AllSemestersDetail`): catalog facts and the semesters offering
  it, no sections.
- No `term`: the last semester visited (`resolveTerm`); when that doesn't offer the course, the nearest semester that
  does; when none does, the all-semesters view. This is the canonical address search engines and shared links use.

The page shows the course facts ("Also listed as" for cross-listings with the same title, "Meets with" otherwise),
adding it to the semester's active schedule with any section or recitation (`CourseSchedule` in
`course-schedule.svelte.ts`, over the `schedules` store, with conflicts against the rest of the schedule), the
requisite status checked against the student's record, plan and schedules (with waivers through
`record.setOverride`), the catalog description and requisites, which of the active degree plan's requirements it
counts toward, the semesters offering it, FCE summary, trend charts and history tables, past syllabi (PDF preview in a
dialog, downloads), and the prerequisite tree beside what it unlocks. The schedules feature's `WeekGrid` will sit
beside the sections; the page leaves a slot for it.

Signed out, public term documents have no instructors or rooms, so the sections say "Sign in to see instructors and
rooms" once; FCEs, syllabi and requirement matches show a sign-in prompt (`MemberSignIn`) in place of the data.

"Back to plan" appears when the student came from the degree plan (`planOrigin()`); otherwise the back link returns to
the course list as it was. `CourseHead.svelte` sets the title, a meta description from the catalog description and
the canonical `/course/<code>` link; the web API renders the same tags on the server
([decision 25](../decisions/0025-render-course-head-tags-on-the-server.md)).

`CourseDetailPanel.svelte` stacks the same sections into a side panel for the Courses page (`?course=` there). Course
links inside a view go through `setCourseHref()` (`links.ts`), so the panel can open them in place.

## Requisite explorer

`/requisites/[code]?term=` ports Courseweave's `RequisitesPage.tsx`: a course's prerequisite tree (expandable, with
cycles marked) and the courses it unlocks, grouped by department, with the length of its shortest prerequisite chain.
Every course link re-centers the explorer. Without a course it lists the gateway courses, the ones the most other
courses require. The search box ranks the semester's courses first, then the rest of the catalog.
