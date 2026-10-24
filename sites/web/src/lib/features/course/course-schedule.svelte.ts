// One course against the student's active schedule for a semester: its entry, adding it, switching sections, the
// conflicts each choice would cause, and the week calendar with this course placed or previewed (Courseweave's
// useCourseSchedule).
import { isUnscheduled } from "$lib/core/blocks";
import { nextColor } from "$lib/core/colors";
import type { Term } from "$lib/core/data";
import {
  childForSection,
  initialChoice,
  joinList,
  partBlocks,
  partLabel,
} from "$lib/core/planner/format";
import { resolveSelection, selectionParts, type Selection } from "$lib/core/sections";
import type { ScheduleView } from "$lib/core/state/schedule-view";
import type { Course, Section } from "$lib/core/types";
import type { CalendarBlock } from "$lib/core/week-grid";
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

  /**
   * The section row the student is pointing at, focused in, or (on a touch screen) tapped: the calendar previews
   * what acting on it would give.
   */
  hovered = $state.raw<Section | null>(null);

  /** Previews a row's result in the calendar, or stops when `section` is null. */
  preview(section: Section | null): void {
    this.hovered = section;
  }

  /** The course's color: its schedule entry's, else the first one the schedule doesn't use. */
  get color(): number {
    return this.entry?.entry.color ?? nextColor(this.view.entries.map((e) => e.entry.color));
  }

  /** Sections of this course on the calendar now (none while it's hidden). */
  readonly #placedParts = $derived.by((): Section[] => {
    const entry = this.entry;
    return entry !== null && !entry.entry.hidden ? entry.parts : [];
  });

  /** What the calendar previews: the hovered row's result, else (course not placed) the section it would add with. */
  readonly #preview = $derived.by(() => {
    const course = this.course;
    // A row of a course shown before (the panel switches courses in place) doesn't count.
    const hovered =
      this.hovered !== null && course?.sections.includes(this.hovered) === true
        ? this.hovered
        : null;
    let selection: Selection | null = null;
    if (course !== null && hovered !== null) selection = this.resultFor(hovered);
    else if (course !== null && this.#placedParts.length === 0) {
      const first = initialChoice(course, this.view);
      selection =
        first.section === null ? null : resolveSelection(course, first.section, first.child);
    }
    const all = selection === null ? [] : selectionParts(selection);
    const placed = this.#placedParts;
    const conflicting = this.#others.filter((o) => partsConflict(all, o.parts));
    return { all, parts: all.filter((p) => !placed.includes(p)), conflicting };
  });

  /** The rest of the schedule, this course's sections, and a hatched preview of the hovered (or default) choice. */
  readonly blocks = $derived.by((): CalendarBlock[] => {
    const course = this.course;
    if (course === null) return [];
    const preview = this.#preview;
    const showing = preview.parts.length > 0;
    const previewConflicts = new Set(preview.conflicting.map((o) => o.course.id));
    const color = this.color;
    return [
      ...this.#others.flatMap((o) =>
        o.parts.flatMap((p) =>
          partBlocks(o.course, p, o.entry.color, {
            conflict:
              previewConflicts.has(o.course.id) ||
              (!showing && (this.view.conflicts.get(o.course.id)?.includes(course.id) ?? false)),
          }),
        ),
      ),
      ...this.#placedParts.flatMap((p) =>
        partBlocks(course, p, color, {
          selected: this.hovered === p,
          conflict: !showing && this.view.conflicts.has(course.id),
        }),
      ),
      ...preview.parts.flatMap((p) =>
        partBlocks(course, p, color, {
          variant: "preview",
          keyPrefix: "preview-",
          conflict: previewConflicts.size > 0,
        }),
      ),
    ];
  });

  /** One line under the calendar saying what it shows. */
  readonly caption = $derived.by((): string => {
    const preview = this.#preview;
    const placed = this.#placedParts;
    const label = preview.all.map((p) => partLabel(p)).join(" and ");
    if (preview.parts.length > 0) {
      if (preview.parts.every(isUnscheduled))
        return `${label} ${preview.all.length > 1 ? "don't" : "doesn't"} have a scheduled meeting time.`;
      if (preview.conflicting.length > 0)
        return `${label} would conflict with ${joinList(preview.conflicting.map((o) => o.course.code))}.`;
      return `Previewing ${label} in your schedule.`;
    }
    if (placed.length > 0)
      return `${placed.map((p) => partLabel(p)).join(" and ")} ${placed.length > 1 ? "are" : "is"} in your schedule.`;
    return "";
  });

  /** The caption warns about a conflict. */
  get captionIsConflict(): boolean {
    return this.#preview.parts.length > 0 && this.#preview.conflicting.length > 0;
  }

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
