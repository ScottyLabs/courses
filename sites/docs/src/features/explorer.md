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
| `ExplorerRail.svelte` | The filter rail: saved only, fits my schedule, department, units (a range, and Courseweave's variable units as "Variable units only", `units=var`), offered in, level, requirement (gen eds or any program requirement, members only), days and times, session, delivery, scheduled only, FCE hours and rating, how cards aggregate FCEs, and courses not offered |
| `ExplorerToolbar.svelte`, `chips.ts` | Search, result count, removable filter chips, semester, sort, the FCE chart and the cards/list switch |
| `CourseResults.svelte` | Cards (20 at a time), rows (100 at a time) or the hours chart; more load as the end of the list scrolls into view |

Selecting a course opens it in the detail panel (`?course=15-122`) beside the list from 1024px wide. Below 1440px the
filter rail collapses to its strip while a course is open (`RailLayout`'s `autoCollapse`); showing the rail again puts
the course in place of the list. Below 1024px the course replaces the list. The panel's Expand goes to the full course page. "/" focuses the search, the arrow keys move through
the results from the search field and Enter opens the highlighted one.

Signed out, the requirement filter, FCE filters and the FCE parts of cards are replaced by sign-in prompts; the list,
search and the other filters work on the public documents.

## Shared course components

`sites/web/src/lib/components/course/` holds what the Courses page, the profile, schedules and instructor pages
share. Each component needs only a course (or its id) and a semester:

- `CourseCard` (the Figma card: requisite status with a popover of the expression as status chips, semester select,
  save star, friends taking it; expanded, the FCE aggregate and table), `CourseRow` in a `CourseList` (Courseweave's
  compact rows) and `CourseCardSmall` (the Figma's small card, for saved courses and the record's Courses Taken and
  Currently Taking, with `deptLabel`, `children` and `footer` slots).
- `RequisiteStatus` (the requisite line with its details and waivers, on the course page, the panel and the schedule
  rail's checks), `RequisiteLevelIcon`, `RequisiteExpression`, `RequisiteSummary`, `RequisiteTreeNode`,
  `RequiresAndUnlocks`, `UnlocksList`, `CourseChipList`.
- `FceAggregate`, `FceStats` (the Figma's stat boxes), `FceTable` (with optional course and instructor columns,
  `showCourse`, `showInstructor`), `FceAggregateSettings` (the rails' "Aggregate FCEs" group: one setting,
  `cmucourses:fce-aggregate`, shared by the Courses and Instructors rails), `CourseFcePanel`, `FceHistory`,
  `LineChart`, `HoursScatter`, `WorkloadMeter`, `InstructorFceBadge`; `fce-aggregate.ts` and `fce-format.ts`.
- `SaveButton`, `FriendsTaking` (from `GET /api/friends/courses?term=`), `MemberSignIn`.
- `course-search.ts` (catalog search for the plan's "Add a course", the record's course picker and the requisite
  explorer) and `links.ts` (`setCourseHref()`, so the Courses page's panel opens course links in place).

Course previews ([decision 41](../decisions/0041-preview-every-course-link.md)): `CoursePreviewLayer`, mounted once in
the root layout, shows a card about a course (units, title, description, prerequisites, the student's requisite status,
the semester's meetings, FCE hours) when the pointer rests on or keyboard focus reaches any link to a course page, the
requisite explorer or the Courses page's panel, in the link's `term`; on a touch screen, pressing and holding the link
opens it at the bottom of the screen with "Open course". Codes that aren't links opt in with `coursePreview(id, term)`
from `preview.ts` (or `attrs` on `Chip`); a course's own card or row opts out with `data-course-preview="off"`.

They load what they show through `data.svelte.ts` (catalog index, a semester, offerings, descriptions, FCEs,
syllabi, friends taking a course) and `requisite-context.svelte.ts` (the student's requisite context for a
semester), which load each document once however many components ask
([decision 32](../decisions/0032-share-course-data-between-components.md)). `fce-options.svelte.ts` remembers how
cards aggregate FCEs on this device.

The profile's Saved Courses section (`lib/features/profile/SavedSection.svelte`) lists saved courses as small cards or
rows, with department and level filters in the section's title band (`profileBand()`).

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
`record.setOverride`; while the record is empty, a missing prerequisite asks for the courses taken instead of
showing an error, and status chips leave "missing" uncolored), the catalog description and requisites, which of the active degree plan's requirements it
counts toward, the semesters offering it, FCE summary, trend charts and history tables, past syllabi (PDF preview in a
dialog, downloads), and the prerequisite tree beside what it unlocks. Beside the sections (from 1024px; before them on
narrower screens) the semester's active schedule shows as a week calendar (`ScheduleCalendar.svelte`, on the shared
`WeekGrid`): the course placed, or previewed hatched while the student points at, focuses or (on a touch screen) taps a
section row, else the section it would be added with; a line under it says what it shows and names conflicts.

Signed out, public term documents have no instructors or rooms, so the sections say "Sign in to see instructors and
rooms" once; FCEs, syllabi and requirement matches show a sign-in prompt (`MemberSignIn`) in place of the data.

The back link returns to the page the course was opened from (`originBack()` in `state/nav-origin.ts`): "Back to
plan", "Back to schedule", "Back to courses" (the list as it was), "Back to instructor(s)", "Back to profile" or "Back
to requisites"; opened fresh, it goes to the semester's course list. `CourseHead.svelte` sets the title, a meta description from the catalog description and
the canonical `/course/<code>` link; the web API renders the same tags on the server
([decision 25](../decisions/0025-render-course-head-tags-on-the-server.md)).

`CourseDetailPanel.svelte` stacks the same sections into a side panel for the Courses page (`?course=` there), with
the calendar before the sections. Course
links inside a view go through `setCourseHref()` (`links.ts`), so the panel can open them in place.

## Requisite explorer

`/requisites/[code]?term=` ports Courseweave's `RequisitesPage.tsx`: a course's prerequisite tree (expandable, with
cycles marked) and the courses it unlocks, grouped by department, with the length of its shortest prerequisite chain.
Every course link re-centers the explorer. Without a course it lists the gateway courses, the ones the most other
courses require. The search box ranks the semester's courses first, then the rest of the catalog.
