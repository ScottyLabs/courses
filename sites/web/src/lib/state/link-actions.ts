// Plan edits that may land in a schedule (Courseweave's features/degree/linkActions.ts): a semester synced with a
// schedule (PlanTerm.link) shows that schedule's courses, so adding, removing, moving and pass/fail changes go to the
// schedule. The plan's own list for that semester is kept as a snapshot (the fallback if the schedule is deleted,
// and unit overrides).
import type { Term } from "../core/data.ts";
import { initialChoice } from "../core/planner/format.ts";
import {
  effectiveTermCourses,
  linkedCourseId,
  linkedScheduleId,
  parseLinkedCourseId,
  type Plan,
  type PlanCourse,
} from "../core/plan.ts";
import type { SnapshotCourse } from "../core/state/plan.ts";
import { buildScheduleView } from "../core/state/schedule-view.ts";
import type { Schedule } from "../core/state/store.ts";
import type { PlanStore } from "./plans.ts";
import type { ScheduleStore } from "./schedules.ts";

export interface LinkStores {
  schedules: ScheduleStore;
  plans: PlanStore;
  /** A semester's course data, to pick sections for a course added to a schedule. */
  loadTerm(id: string): Promise<Term>;
}

function termOfSchedule(plan: Plan, scheduleId: string): string | undefined {
  return plan.terms.find((t) => t.link?.scheduleId === scheduleId)?.term;
}

/** Snapshot rows for a schedule's visible courses, carrying unit overrides from the plan's own list. */
function scheduleSnapshot(schedule: Schedule, planCourses: PlanCourse[]): SnapshotCourse[] {
  return schedule.entries
    .filter((e) => !e.hidden)
    .map((e) => ({
      courseId: e.courseId,
      passFail: e.passFail,
      units: planCourses.find((c) => c.courseId === e.courseId)?.units,
    }));
}

/** Plan and schedule edits for the degree planner. Every method takes the plan as the page shows it. */
export class LinkActions {
  readonly #schedules: ScheduleStore;
  readonly #plans: PlanStore;
  readonly #loadTerm: (id: string) => Promise<Term>;

  constructor(stores: LinkStores) {
    this.#schedules = stores.schedules;
    this.#plans = stores.plans;
    this.#loadTerm = (id) => stores.loadTerm(id);
  }

  /** The plan as stored now (the page's copy may be a render behind). */
  #current(plan: Plan): Plan {
    return this.#plans.get(plan.id) ?? plan;
  }

  /** Adds a course to a schedule, then picks a conflict-free section once the term's course data is loaded. */
  #addToSchedule(scheduleId: string, term: string, courseId: string, passFail?: boolean): void {
    const schedules = this.#schedules;
    const existing = schedules.get(scheduleId)?.entries.find((e) => e.courseId === courseId);
    schedules.addCourse(scheduleId, courseId);
    if (existing !== undefined) return;
    if (passFail === true) schedules.setPassFail(scheduleId, courseId, true);
    this.#loadTerm(term).then(
      (t) => {
        const s = schedules.get(scheduleId);
        const entry = s?.entries.find((e) => e.courseId === courseId);
        const course = t.courseById.get(courseId);
        if (s === undefined || entry?.section !== null || course === undefined) return;
        const others: Schedule = {
          ...s,
          entries: s.entries.filter((e) => e.courseId !== courseId),
        };
        const choice = initialChoice(course, buildScheduleView(others, t));
        if (choice.section !== null)
          schedules.setSection(scheduleId, courseId, choice.section, choice.child);
      },
      () => {
        // No course data for the term: the student picks a section later.
      },
    );
  }

  /** Adds a course to a plan semester (or its synced schedule). Returns the plan-course id to focus. */
  addToTerm(
    plan: Plan,
    term: string,
    courseId: string,
    options: { passFail?: boolean; units?: number } = {},
  ): string {
    const scheduleId = linkedScheduleId(this.#current(plan), term, this.#schedules.items);
    const planned = this.#plans.addCourse(plan.id, term, courseId, options);
    if (scheduleId === null) return planned;
    this.#addToSchedule(scheduleId, term, courseId, options.passFail);
    return linkedCourseId(scheduleId, courseId);
  }

  /**
   * Puts a course just before another course of its semester (null: at the end). Order is only for organizing. A
   * synced semester's order is its schedule's, so that's what moves.
   */
  placeInTerm(plan: Plan, planCourseId: string, term: string, before: string | null): void {
    if (before === planCourseId) return;
    const linked = parseLinkedCourseId(planCourseId);
    if (linked !== null) {
      const beforeCourse = before === null ? null : (parseLinkedCourseId(before)?.courseId ?? null);
      this.#schedules.moveEntry(linked.scheduleId, linked.courseId, beforeCourse);
      return;
    }
    const own = this.#current(plan).terms.find((t) => t.term === term)?.courses ?? [];
    const others = own.filter((c) => c.id !== planCourseId);
    const at = before === null ? -1 : others.findIndex((c) => c.id === before);
    this.#plans.moveCourse(plan.id, planCourseId, term, at < 0 ? others.length : at);
  }

  /** Removes a course from the plan, or from the synced schedule it comes from. */
  removeFromPlan(plan: Plan, planCourseId: string): void {
    const linked = parseLinkedCourseId(planCourseId);
    if (linked === null) {
      this.#plans.removeCourse(plan.id, planCourseId);
      return;
    }
    this.#schedules.removeCourse(linked.scheduleId, linked.courseId);
    const term = termOfSchedule(this.#current(plan), linked.scheduleId);
    if (term !== undefined) this.#plans.removeCourseFromTerm(plan.id, term, linked.courseId);
  }

  /** The course behind a plan-course id, wherever it lives. */
  #findCourse(plan: Plan, planCourseId: string): { term: string; course: PlanCourse } | null {
    const eff = effectiveTermCourses(plan, this.#schedules.items);
    for (const t of plan.terms) {
      const course = eff.get(t.term)?.courses.find((c) => c.id === planCourseId);
      if (course !== undefined) return { term: t.term, course };
    }
    return null;
  }

  /**
   * Moves a course to another semester, or within its own. `before` places it just before that course (null: at the
   * end); without it a course moved to another semester goes last and one already there stays put.
   */
  moveInPlan(plan: Plan, planCourseId: string, toTerm: string, before?: string | null): void {
    const found = this.#findCourse(this.#current(plan), planCourseId);
    if (found === null) return;
    if (found.term === toTerm) {
      if (before !== undefined) this.placeInTerm(plan, planCourseId, toTerm, before);
      return;
    }
    const fromLinked = parseLinkedCourseId(planCourseId) !== null;
    const toLinked = linkedScheduleId(this.#current(plan), toTerm, this.#schedules.items) !== null;
    if (!fromLinked && !toLinked) {
      this.#plans.moveCourse(plan.id, planCourseId, toTerm);
      if (before !== undefined && before !== null)
        this.placeInTerm(plan, planCourseId, toTerm, before);
      return;
    }
    this.removeFromPlan(plan, planCourseId);
    const { courseId, passFail, units } = found.course;
    const id = this.addToTerm(plan, toTerm, courseId, { passFail, units });
    if (before !== undefined && before !== null) this.placeInTerm(plan, id, toTerm, before);
  }

  setPlanPassFail(plan: Plan, planCourseId: string, passFail: boolean): void {
    const linked = parseLinkedCourseId(planCourseId);
    if (linked === null) this.#plans.updateCourse(plan.id, planCourseId, { passFail });
    else this.#schedules.setPassFail(linked.scheduleId, linked.courseId, passFail);
  }

  setPlanUnits(plan: Plan, planCourseId: string, units: number | undefined): void {
    const linked = parseLinkedCourseId(planCourseId);
    if (linked === null) {
      this.#plans.updateCourse(plan.id, planCourseId, { units });
      return;
    }
    const term = termOfSchedule(this.#current(plan), linked.scheduleId);
    if (term !== undefined) this.#plans.setCourseUnits(plan.id, term, linked.courseId, units);
  }

  /**
   * Syncs a plan semester with a schedule. `merge` first adds the semester's planned courses that the schedule
   * lacks; otherwise the schedule is used as is (those planned courses are dropped from the semester).
   */
  linkTermToSchedule(plan: Plan, term: string, schedule: Schedule, mode: "merge" | "as-is"): void {
    const own = this.#current(plan).terms.find((t) => t.term === term)?.courses ?? [];
    const snapshot = scheduleSnapshot(schedule, own);
    if (mode === "merge") {
      const have = new Set(snapshot.map((c) => c.courseId));
      for (const c of own) {
        if (have.has(c.courseId)) continue;
        have.add(c.courseId);
        this.#addToSchedule(schedule.id, term, c.courseId, c.passFail);
        snapshot.push({ courseId: c.courseId, passFail: c.passFail, units: c.units });
      }
    }
    this.#plans.linkTerm(plan.id, term, schedule.id, snapshot);
  }

  /** Stops syncing; the semester keeps the schedule's current courses (or its snapshot if the schedule is gone). */
  unlinkTerm(plan: Plan, term: string): void {
    const eff = effectiveTermCourses(this.#current(plan), this.#schedules.items).get(term);
    if (eff === undefined || eff.schedule === null) {
      this.#plans.unlinkTerm(plan.id, term);
      return;
    }
    const courses = eff.courses.map((c) => ({
      courseId: c.courseId,
      passFail: c.passFail,
      units: c.units,
    }));
    this.#plans.unlinkTerm(plan.id, term, courses);
  }
}

/** The degree planner's link actions over these stores. */
export function linkActions(stores: LinkStores): LinkActions {
  return new LinkActions(stores);
}
