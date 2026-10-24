# Degree plan and academic record

## Degree plan

`/plan` ports Courseweave's degree planner (`src/features/degree/*`). The route is a thin wrapper around
`PlanPage.svelte` in `sites/web/src/lib/features/plan/`; the logic is Courseweave's, already in `lib/core`
(`plan.ts`, `audit.ts`, `plan-table.ts`, `requirement-match.ts`, `substitutes.ts`, `degree/progress.ts`,
`degree/hooks.ts`), and the plan's data is the `plans`, `links`, `schedules`, `record` and `preferences` stores.

Which plan it shows: `?plan=<id>` (`planPath()`), else the active plan (`plans.activeId`: the one chosen, else the most
recently edited). Opening a plan makes it the active one. With no plan yet, the page is the first-run start
(`StartPlan.svelte`): the first semester (from the record's entry term when there is one), programs, "Create plan",
and "Import from Stellic". The semester being browsed for titles, units and "offered in" hints is `?term=`, else the
last one visited (`resolveTerm`).

The layout follows Courseweave's plan page in the design system's look
([decision 36](../decisions/0036-lay-out-the-degree-plan-beside-its-requirements.md)):

- **Header** (`PlanHeader.svelte`): the plan picker, "New plan" (same programs and start), and the plan's options:
  rename, duplicate, send to Stellic (a script for the student's own Stellic tab, `core/stellic-export.ts`), import
  from Stellic, delete. Then the programs (`audit/ProgramPicker.svelte`: search with the kind filter for majors,
  minors, additional majors and concentrations, each with its requirements version by entry year and campus,
  optional concentrations, and what's checked alongside), and the numbers: units planned (of the degree's total),
  units completed, requirements met or planned, semesters with the start term, and the overall progress bar
  (`DegreeProgressBar.svelte`: done, in progress, planned, still needed).
- **Record prompt** while the record is empty, and **requisite problems**: courses planned before their requisites,
  each jumping to the course.
- **Semester grid** (`SemesterGrid.svelte`): incoming credit (AP, IB, transfer, placements), then academic years with
  fall, spring and, when planned, summer (`grid.ts`), slots to add back a removed semester, the next semester and
  summers. A cards or detailed layout (the `planLayout` preference); the detailed one is a table per semester with
  the requirements each course satisfies, when it's usually offered and its prerequisites colored against the plan.
- **Semester cards** (`TermCard.svelte`): record courses (read-only, with grades) and planned courses, each with
  pass/fail, offering warnings, the requisite state (its issues and "Mark requisites as waived" on click), FCE hours,
  units (editable for variable-unit courses) and a menu: move up or down, move to another semester, pass/fail, units,
  view, remove. "Add a course" searches the whole catalog and says whether the semester offers each result. The
  footer has factorable and pass/fail units, FCE hours a week (at the busier half when minis run) and load guidance.
  The semester's menu syncs it with one of its schedules (`links` store: the semester then shows the schedule's
  courses, and edits go to the schedule, with a section picked for added courses), copies a schedule's courses,
  opens the semester on the schedules page, or removes it.
- **Requirements** (`RequirementsPanel.svelte`, `audit/*`): each program's state, areas done and unit progress, then
  the requirement tree with done, planned, in progress and still needed states, the courses counted toward each,
  "ways to satisfy" a pool with Add, rules the audit can't check, and free electives. "Add" puts a course in the
  semester `suggestTerm()` picks (offered, with room, prerequisites met by then) and jumps to it. "Expand" opens the
  requirements in a dialog.

From 1180px the requirements are a sticky column beside the semesters; below that the two are tabs. Drag and drop
(HTML drag events, `DRAG_TYPE` and `DRAG_NEW_COURSE`) moves courses between and within semesters and adds courses
from the requirements; it's on with a fine pointer from 768px. The course menus and "Add" do the same from the
keyboard or a touch screen. On phones, past semesters start collapsed.

Course links go to `/course/<code>?term=<semester>` (`courseHref()`), so the course page offers "Back to plan"
(`planOrigin()`).

Signed out, plans are drafts in the tab, saved to the account on signing in, and the program picker and requirements
say degree programs need a CMU sign-in
([decision 37](../decisions/0037-let-signed-out-visitors-draft-a-degree-plan.md)).

Pure helpers with tests: `grid.ts` (year rows), `rows.ts` (a course row's facts, where a drop lands, nudging),
`course-search.ts` (catalog search for "Add a course") and `audit/programs.ts` (program search and labels).

## Academic record

The academic record lives in the profile, ported from Courseweave's record page (`src/features/record/*`): the
components are in `sites/web/src/lib/features/record/`, the logic in `lib/core` (`sio-record.ts`, `stellic-plan.ts`,
`exam-credit.ts`, `core/record/*`, `core/state/record.ts`) and the data in the `record` store.

- **Courses Taken** (`features/profile/TakenSection.svelte`) is the whole record, in Courseweave's order: the actions
  (import from SIO or Stellic, export and import a backup in Courseweave's JSON format, with merge or replace, and
  clear the record), the empty-record start with the import and links to add by hand, the student's college, first
  semester and expected graduation beside the unit totals and QPA, then the courses taken, exam credit, transfer credit,
  and placements and requisite overrides. Courses taken look like the Figma's Courses Taken page: department, course
  level and semester filters, and cards (with grade and semester) or a list (`ViewToggle`, `profile-taken`), grouped
  by semester; grades can be changed and courses removed in both. The add form searches the catalog and the browsed
  semester, checks units, and takes a pasted list of course numbers. Exam credit picks an AP, IB or A-Level exam and
  score and shows what CMU awards from the registrar's charts, checking college eligibility, with a form for credit the
  charts don't cover.
- **Currently Taking** (`TakingSection.svelte`) is this semester's courses from the record, with their sections and
  meeting times from the semester's active schedule, as cards or a list with department and level filters. When the
  active schedule has courses the record lacks, it offers to add them.
- **Import** (`SioImportButton.svelte`, `ImportReview.svelte`): an SIO unofficial academic record PDF or a Stellic plan
  PDF is read in the browser by pdf.js, which loads in its own chunk only when a file is chosen
  (`core/record/SioImportPdf.ts`); the file never leaves the device
  ([decision 17](../decisions/0017-store-personal-data-on-the-server.md)). The review lists what was read by group,
  with what's already in the record unchecked, an option to update the profile, and the lines that couldn't be read.
  A Stellic plan also becomes a new degree plan. The degree plan's first run and its options menu use the same button.
- **Onboarding** (`RecordOnboarding.svelte`, mounted once in the root layout): new students are asked once, after their
  account's data has loaded, to import their record or add it by hand (`personal.onboardingDue`,
  `personal.markOnboardingAsked()`); not on the Courses Taken page itself. `RecordPrompt.svelte` nudges again on the
  degree plan while the record is empty, until it's hidden (`preferences.hideOnboardingPrompt()`).
- **Requisite status** (`RequisiteStatus.svelte`): the requisite line with its details and waivers, for the course and
  schedule pages.

Pure helpers with tests: `core/record/taken-list.ts` (which entries each list shows, filters, grouping, the courses
taken this semester) and `core/record/grades.ts` (grade choices and notes).
