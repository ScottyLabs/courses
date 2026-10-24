# Schedules and finals

The schedule planner is Courseweave's (`src/features/planner`, `src/features/schedule-tools`), on the Figma's
schedule page: the left rail, a toolbar band with the schedule's name and total workload, and the week calendar.
Personal data goes through the `schedules`, `preferences`, `record`, `plans` and `saved` stores (`$lib/state`); the
pure logic is in `$lib/core` (`state/schedule-view.ts`, `planner/format.ts`, `generator.ts`, `ics.ts`, `share.ts`,
`stellic-export.ts`).

## The week calendar

`WeekGrid` (`lib/components/calendar/`) is the calendar every page uses: the schedules page, the course page and
instructor pages. It takes `CalendarBlock`s (`core/week-grid.ts`, built with `sectionBlocks` in `core/blocks.ts` or
`partBlocks` in `core/planner/format.ts`) and reports clicks and hovers through each block's `onClick`/`onHover` and
the `onblockclick`/`onblockhover` props.

| Prop | Meaning |
| --- | --- |
| `blocks` | What to draw. `variant` solid (in the schedule), preview (hatched), ghost (an outlined alternative); `conflict`, `selected`, `mini` |
| `days`, `minHour`, `maxHour`, `hourHeight` | Defaults: Monday to Friday plus weekend days with a block, 8 AM to 6 PM widened to fit |
| `display` | What blocks show besides the code; defaults to the student's calendar display preferences |
| `view` | `auto` (one day at a time below 640px with a Day/Week switch, remembered on the device), `day` or `week` |
| `tba`, `tbaTitle` | Things without a set time, listed under the grid |
| `compact`, `label`, `empty` | Small previews in dialogs; the accessible name; content over an empty grid |

Overlapping blocks share their column side by side. A visually hidden list repeats the calendar day by day for screen
readers; clickable blocks are buttons named with the course, day, time and room. Course colors are the ten palette
hues in `routes/layout.css` (`--c0` to `--c9`, with dark values), applied with `courseColorStyle()` and the
`course-tint`, `course-swatch` and `hatch` utilities. `calendarPng()` draws a schedule on a canvas for "Save as image".

## The schedules page

`/schedules?term=<id>` shows the semester's schedules as tabs (`ScheduleTabs.svelte`): switch, create (blank or from
a degree plan semester, `core/planner/fromPlan.ts`), rename (the pencil, double-click or the menu), duplicate, share
and delete. A semester without a schedule shows "Schedule 1", created on the first edit, so visiting creates nothing.
`?schedule=<id>` shows a particular one (the profile links to it).

- **Toolbar**: the course search (`AddCourseSearch.svelte`: arrowing or hovering previews a course's sections, Enter
  adds it with sections that fit), removable course chips, Generate, Finals, Export and Show (the calendar display
  menu), then the total units, expected hours a week from FCEs (signed in), the factorable and pass/fail split, the
  over-54-units warning and the conflict count. On phones Generate, Finals, Export and Show are icon buttons beside
  the "Courses" sheet button, and the chips get their own row (`PageToolbar`'s `actionsLayout`).
- **Rail**: the semester, the courses (`CourseEntryRow.svelte`: color, lecture and recitation pickers with times,
  rooms, instructors' FCEs and conflicts, pass/fail, hide, remove, move up or down), saved courses offered this
  semester with quick add, the requisite check against the record (`ScheduleRequisiteSummary.svelte`, built on
  `core/record/context.ts`), the FCE workload by course and by half of the semester, and a friend's schedule to
  compare. At phone width the rail is the "Courses" sheet.
- **Calendar**: clicking a course outlines its other lectures (or recitations); clicking an outline switches to it.
  A session switch appears when a course runs in a mini. A friend's friends-visible schedule
  (`GET /api/friends/{id}/schedules?term=`) shows as striped blocks with their name.
- **Generator** (`ScheduleGenerator.svelte`): every conflict-free arrangement, ranked by start and end times, days
  off, gaps, lunch and days on campus, with courses left out or locked; apply it or save it as a new schedule.
- **Export** (`ExportMenu.svelte`): a .ics file, a PNG of the week, the course list as text, sharing, and "Send to
  Stellic or SIO" (`components/stellic/SendToStellicDialog.svelte`). That dialog only hands the student a script to run in their own
  signed-in Stellic or SIO tab, which shows what it will add and changes nothing until they confirm; this site never
  talks to Stellic or SIO. It takes `mode="plan"` for a degree plan.
- **Sharing** (`ShareDialog.svelte`, [decision 35](../decisions/0035-share-schedules-by-token-or-by-a-link-that-carries-it.md)):
  who among friends sees the schedule, and a link. `/schedules/shared/<token>` is the read-only view with "Copy to my
  schedules"; `?import=<payload>` links open in `ImportLinkDialog.svelte`.

Signed out, the page works on a draft kept in the tab, with a notice that it isn't saved; rooms, instructors and
hours a week ask the student to sign in.

## Final exams

Final exams are a dialog on the schedules page (`features/finals/FinalsDialog.svelte`,
[decision 34](../decisions/0034-show-final-exams-in-a-dialog-on-each-schedule.md)), opened with Finals or
`/schedules?term=<id>&finals=1`; `/finals` redirects there. It shows the schedule's exams matched by section
(`matchFinals` in `finals.ts`), warnings for overlaps, back-to-back exams and three or more within 24 hours, a .ics
download, a search of the semester's whole registrar schedule (`finals-table.ts`), a notice while the registrar
hasn't published it, and the SIO calendar import (`SioImport.svelte`) for exams its file lacks.

## Profile

`/profile/schedules` (`features/profile/SchedulesSection.svelte`) lists every schedule as a card with its semester,
units, hours a week and courses, filtered by semester, each opening the schedule.
