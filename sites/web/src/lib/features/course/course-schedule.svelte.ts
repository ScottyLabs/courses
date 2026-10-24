// One course against the student's active schedule for a semester: its entry, adding it, switching sections and
// the conflicts each choice would cause. Courseweave's useCourseSchedule, without the calendar preview (the schedules
// feature's WeekGrid will show that).
import type { Term } from "$lib/core/data";
import { childForSection, initialChoice } from "$lib/core/planner/format";
import { resolveSelection, selectionParts, type Selection } from "$lib/core/sections";
import type { ScheduleView } from "$lib/core/state/schedule-view";
import type { Course, Section } from "$lib/core/types";
import { buildScheduleView, partsConflict, schedules, unitTotals } from "$lib/state";

/** The course and the semester it's shown in; null while either is missing. */
export type ScheduleSource = () => { course: Course; term: Term } | null;

const EMPTY_VIEW: ScheduleView = {
  schedule: null,
  entries: [],
  placed: [],
  conflicts: new Map(),
  units: unitTotals([]),
  hasCourse: () => false,
};
const NO_SELECTION: Selection = { section: null, child: null, needsChild: false };

export class CourseSchedule {
  readonly #source: ScheduleSource;

  constructor(source: ScheduleSource) {
    this.#source = source;
  }

  get course(): Course | null {
    return this.#source()?.course ?? null;
  }

  get termId(): string | null {
    return this.#source()?.term.meta.id ?? null;
  }

  /** The schedule shown for the semester, or null until the student has one. */
  readonly scheduleId = $derived.by(() => {
    const termId = this.termId;
    return termId === null ? null : schedules.activeId(termId);
  });

  readonly view = $derived.by((): ScheduleView => {
    const source = this.#source();
    if (source === null) return EMPTY_VIEW;
    const id = this.scheduleId;
    return buildScheduleView(id === null ? null : (schedules.get(id) ?? null), source.term);
  });

  readonly entry = $derived.by(() => {
    const id = this.course?.id;
    return id === undefined ? null : (this.view.entries.find((e) => e.course.id === id) ?? null);
  });

  readonly #others = $derived.by(() => {
    const id = this.course?.id;
    return this.view.placed.filter((e) => e.course.id !== id);
  });

  /** The schedule's name, for "Add to Schedule 1". */
  get scheduleName(): string | null {
    return this.view.schedule?.name ?? null;
  }

  /** What the schedule would hold for this course after acting on a section's row. */
  resultFor(section: Section): Selection {
    const course = this.course;
    if (course === null) return NO_SELECTION;
    if (section.parent !== null) return resolveSelection(course, section.parent, section.name);
    const entry = this.entry;
    const keepChild =
      entry?.section?.name === section.name
        ? (entry.child?.name ?? null)
        : childForSection(course, section.name, this.view);
    return resolveSelection(course, section.name, keepChild);
  }

  /** Codes of scheduled courses the result of acting on this row would conflict with. */
  conflictsFor(section: Section): string[] {
    const parts = selectionParts(this.resultFor(section));
    return this.#others.filter((o) => partsConflict(parts, o.parts)).map((o) => o.course.code);
  }

  #target(): string | null {
    const termId = this.termId;
    if (termId === null) return null;
    return this.scheduleId ?? schedules.ensureSchedule(termId);
  }

  /** Adds the course with the first section (and recitation) that fits. */
  addDefault(): void {
    const course = this.course;
    const id = this.#target();
    if (course === null || id === null) return;
    const choice = initialChoice(course, this.view);
    schedules.addCourse(id, course.id, choice.section, choice.child);
  }

  /** Adds the course with this section, or switches to it. */
  choose(section: Section): void {
    const course = this.course;
    const result = this.resultFor(section);
    const id = this.#target();
    if (course === null || id === null || !result.section) return;
    if (this.entry)
      schedules.setSection(id, course.id, result.section.name, result.child?.name ?? null);
    else schedules.addCourse(id, course.id, result.section.name, result.child?.name ?? null);
  }

  remove(): void {
    const course = this.course;
    if (course !== null && this.scheduleId !== null)
      schedules.removeCourse(this.scheduleId, course.id);
  }
}
