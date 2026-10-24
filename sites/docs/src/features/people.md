# Instructors, friends and settings

The website's people pages: the instructor explorer, friends on the profile, the invite page and the profile's
settings. They follow the [Figma redesign](https://www.figma.com/design/vsvdhGC2QP5fsvG0Pl7W6v/CMUCourses-Redesign--summer-25---Copy-?node-id=502-10393)
and port Courseweave's instructor explorer ([Courseweave features](../courseweave.md)).

## Instructors

`/instructors?term=S27` lists who teaches in a semester; `term=all` is "All semesters", everyone teaching in any
built semester plus everyone in the FCE history. `/instructors/<slug>?term=` is one instructor. Both are member data:
signed out, the pages ask the visitor to sign in, and a non-CMU account is told it needs one.

| Where | What |
| --- | --- |
| `routes/instructors/` | A layout and two thin pages over the feature components |
| `lib/features/instructors/InstructorsShell.svelte` | The layout: filter rail, search toolbar, and the rows beside an open instructor |
| `lib/features/instructors/scope.svelte.ts` | State the pages share: semester, filters (in the URL), documents, results, the open instructor |
| `lib/features/instructors/InstructorResults.svelte` | `/instructors`: cards or rows |
| `lib/features/instructors/InstructorDetail.svelte` | `/instructors/<slug>`: aggregate data, evaluations by section, teaching, evaluation history |
| `lib/features/instructors/fce-aggregate.ts` | Loading an instructor's FCE rows, the aggregate window and figures (tested) |
| `lib/features/instructors/aggregate-settings.svelte.ts` | The rail's "Aggregate FCEs" choices, kept on the device |
| `lib/components/instructor/` | Instructor card and row, FCE stat boxes and table, the trend chart |

The layout keeps the list's state while the student opens instructors. In the list view on screens 1280 pixels wide
or more, an open instructor shows beside Courseweave's rows, which stay in their own scrolling column; in the cards
view and on narrower screens the instructor page is on its own, with "All instructors" going back to the list.

The list keeps its filters in the URL (`q`, `dept`, `saved=1`, `sort=sections`), so Back from an instructor, a
reload or a shared link shows the same list. Search is Courseweave's: names first, then course numbers (`15-122`,
`15-1`) and course titles; `/` focuses the field, the arrow keys move through the rows and Enter opens one. Saved is
a filter, matching instructors who teach one of the student's saved courses. The cards/list switch is remembered per
device; rows have letter headings when sorted by name.

Cards follow the Figma: the name and departments, three stat boxes (workload, teaching rate, course rate) and the
FCE rows the aggregate counts. A card loads its FCE rows when it comes near the viewport, from the `fce/courses/<NN>`
documents of the courses the FCE index lists for that person, so a long list only downloads what is seen.

### Aggregate FCEs

The rail's "Aggregate FCEs" settings choose what the stat boxes average: the most recent N semesters with responses
("Semesters to show", empty for all; default 2) in the chosen seasons (default spring and fall). On an instructor's
page, "Courses" narrows it to some of their courses. Means are weighted by responses, as Courseweave's summaries are.
The settings are a device convenience in localStorage (`cmucourses:fce-aggregate`), shared by both pages.

### Instructor page

- The Figma card: aggregate data in the well, then every evaluated section in the window, or every semester with
  "Every semester" on; semester, course, section, workload, ratings, respondents and response rate.
- In a semester: Courseweave's teaching table (course, title, section, meeting times, rooms, co-instructors), class
  hours a week and "Teaches with". The week calendar comes with the schedules page's shared `WeekGrid`.
- In all semesters: what they taught each semester, linking to that semester's view.
- Evaluation history: recent teaching rating, courses evaluated, teaching rating by semester and a per-course table.
