# 41. Preview every course link, by hover or by press and hold

Date: 2026-10-24

## Status

Accepted. Amended the same day: links to instructors preview the same way (see "Instructor links" below), and the
layer is now `PreviewLayer` in `lib/components/preview/`.

## Context

Course codes are everywhere: requisite chips, the degree plan's rows and requirements, schedule rows, friends'
courses, the record, instructors' teaching. Courseweave shows a small card when the pointer rests on a course pill
(title, units, description, prerequisites, the student's requisite status, this semester's meetings and FCE hours), so
students can read a code without leaving the page. Courseweave marks each pill by hand and shows nothing on touch
screens, where a tap simply follows the link. The four feature areas of CMU Courses were built in parallel and render
course links in dozens of components.

## Decision

- One layer, mounted once in the root layout (`CoursePreviewLayer`), listens on the document and previews any link to
  a course page (`/course/15-122`), the requisite explorer or the Courses page's panel (`/?course=15-122`), in the
  link's `term`. Elements that aren't links (a code in a row, a removable chip) opt in with `coursePreview(id)`; an
  element or link marked `data-course-preview="off"` opts out, as do links to the course the page is about.
- Mouse: Courseweave's delays (open after 450 ms, neighbours almost at once after that, close after 160 ms, the card
  stays while the pointer is on it). Keyboard: on focus-visible, Escape closes. The card describes its link for
  screen readers while open.
- Touch: pressing and holding a course link for 450 ms opens the card at the bottom of the screen with "Open course"
  and a close button, instead of the browser's link menu; a tap still follows the link.
- The card reads the shared course data (decision 32), so previewing a course already on screen loads nothing new.

## Consequences

- New course links get previews without any work; links that are a course's own card or row need
  `data-course-preview="off"` so the card doesn't repeat what's on screen
- Course links on touch screens lose the browser's long-press menu (open in a new tab, copy link)
- The card loads the semester's document when the page hasn't, such as on the profile

## Instructor links

Instructors are linked from as many places as courses (sections, the course facts, FCE tables, co-instructors), and
hovering a name to see who it is answers the same need.

- The one layer also previews links to an instructor's page (`/instructors/<slug>`) in the link's `term` (a
  semester, `all`, or none for the semester the page would pick), with the same delays, focus, touch card ("Open
  instructor") and screen reader description; one input serves both, so moving from a course to an instructor
  opens at once. Names that aren't links opt in with `instructorPreview(name, term)`, such as the section picker's
  options; an instructor's own card or row opts out with `data-instructor-preview="off"`, as do links to the
  instructor the page is about.
- The card shows the name, departments, what they teach in the semester (courses with titles, sections) or where
  they teach across semesters, and their FCE aggregate (teaching and course rating with stars, hours, semesters) in
  the window the "Aggregate FCEs" settings choose, as instructor cards do.
- It reads only the compact instructor document the list reads ([decision 43](0043-list-instructors-from-their-own-documents.md)),
  and `instructors/all` when the instructor doesn't teach in the semester, then the `fce/courses/<NN>` documents
  that name the instructor's courses ([decision 39](0039-build-instructor-fce-rows-in-the-browser.md)), each loaded
  once per session and shared with the instructor pages and course pages.
- Instructors are member data: signed out, no instructor link or name previews (none are shown anyway).

Consequences: a first instructor preview costs about 95 KB gzipped (the semester's list) plus one or two FCE
documents of 20 to 100 KB, about what a first course preview costs; a link to someone who doesn't teach in the
semester also loads `instructors/all` (270 KB gzipped), and its touch card opens their all-semesters page, since the
semester's page would find nobody.
