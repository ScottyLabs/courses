# Course explorer

The Courses page (`/`), the course page (`/course/[code]`), the requisite explorer (`/requisites`) and the shared
course components other pages reuse. They port Courseweave's explorer, catalog and FCE features
(`src/features/explorer`, `catalog`, `fce`) onto the Figma's layout.

## Courses page

`sites/web/src/routes/+page.svelte` wires the pieces in `sites/web/src/lib/features/explorer/`:

| File | What it does |
| --- | --- |
| `query.ts` | The page's state in its URL: Courseweave's filter keys (`core/explorer/filters.ts`) plus `term`, `q`, `sort`, `view=chart`, `unoffered`, `course` and the Figma's extra filters: a units range (`ulo`, `uhi`), a course-number level (`num`) and the seasons a course is offered in (`offered`) |
| `url-state.svelte.ts` | Keeps the query and the URL in step: edits replace the history entry after a pause, a semester change or an opened course adds one, and links or Back are read back in |
| `explorer.svelte.ts` | Everything computed from the query: the semester (`resolveTerm`, `term=all` for all semesters), the documents it needs, the compiled requirement filter, the results and the courses other semesters offer |
| `results.ts` | Ranked search (`scoreCourse`), Courseweave's filters and the extra ones, sorting; other semesters' matches and the "Show courses not offered" list |
| `ExplorerRail.svelte` | The filter rail: saved only, fits my schedule, department, units, offered in, level, requirement (gen eds or any program requirement, members only), days and times, session, delivery, scheduled only, FCE hours and rating, how cards aggregate FCEs, and courses not offered |
| `ExplorerToolbar.svelte`, `chips.ts` | Search, result count, removable filter chips, semester, sort, the FCE chart and the cards/list switch |
| `CourseResults.svelte` | Cards (20 at a time), rows (100 at a time) or the hours chart; more load as the end of the list scrolls into view |

Selecting a course opens it in the detail panel (`?course=15-122`): beside the list from 1440px wide, in place of the
list below that. The panel's Expand goes to the full course page. "/" focuses the search, the arrow keys move through
the results from the search field and Enter opens the highlighted one.

Signed out, the requirement filter, FCE filters and the FCE parts of cards are replaced by sign-in prompts; the list,
search and the other filters work on the public documents.

## Shared course components

`sites/web/src/lib/components/course/` holds what the Courses page, the profile, schedules and instructor pages
share. Each component needs only a course (or its id) and a semester:

- `CourseCard` (the Figma card: requisite status with a popover of the expression as status chips, semester select,
  save star, friends taking it; expanded, the FCE aggregate and table), `CourseRow` in a `CourseList` (Courseweave's
  compact rows) and `CourseCardSmall` (the Figma's small card, used for saved courses).
- `RequisiteExpression`, `RequisiteSummary`, `RequisiteTreeNode`, `RequiresAndUnlocks`, `UnlocksList`,
  `CourseChipList`.
- `FceAggregate`, `FceTable`, `CourseFcePanel`, `FceHistory`, `LineChart`, `HoursScatter`, `WorkloadMeter`,
  `InstructorFceBadge`.
- `SaveButton`, `FriendsTaking` (from `GET /api/friends/courses?term=`), `MemberSignIn`.

They load what they show through `data.svelte.ts` (catalog index, a semester, offerings, descriptions, FCEs,
syllabi, friends taking a course) and `requisite-context.svelte.ts` (the student's requisite context for a
semester), which load each document once however many components ask
([decision 32](../decisions/0032-share-course-data-between-components.md)). `fce-options.svelte.ts` remembers how
cards aggregate FCEs on this device.

The profile's Saved Courses section (`lib/features/profile/SavedSection.svelte`) lists saved courses as small cards or
rows, with department and level filters.

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
