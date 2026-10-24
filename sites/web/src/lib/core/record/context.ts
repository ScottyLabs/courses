/* oxlint-disable typescript/strict-boolean-expressions, unicorn/explicit-length-check -- Courseweave's code, ported unchanged (decision 14) so fixes carry over between the two codebases */
import { resolvePlan, type Plan } from "../plan.ts";
import type { CatalogIndex } from "../catalog-types.ts";
import type { Term } from "../data.ts";
import {
  buildEquivalence,
  createContext,
  termOrdinal,
  type CourseCredit,
  type RequisiteContext,
} from "../requisites.ts";
import { sameCourseTitle } from "../substitutes.ts";
import { entryGrade, type RecordData } from "../state/record.ts";
import type { Schedule } from "../state/store.ts";

export interface PlannedCourse {
  courseId: string;
  /** Term code, e.g. "F26". */
  term: string;
}

export interface ContextInput {
  record: RecordData;
  catalog: CatalogIndex | null;
  /** The term being checked, e.g. "S27". */
  term: string;
  /** Courses taken in that same term (the schedule). They satisfy corequisites only. */
  sameTerm?: string[];
  /** Courses planned in other terms (degree planner). Those before `term` count as planned. */
  planned?: PlannedCourse[];
  /** Extra equivalence groups, e.g. a term's cross-listings. */
  equivalentGroups?: string[][];
}

/**
 * Requisite context for checking courses in `term`:
 * - record entries without a term, or from earlier terms, are completed;
 * - record entries from `term` itself count as same-term;
 * - planned courses from earlier terms count as planned; the schedule counts as same-term.
 */
export function buildRequisiteContext(input: ContextInput): RequisiteContext {
  const target = termOrdinal(input.term);
  const credits: CourseCredit[] = [];
  for (const e of input.record.entries) {
    if (!e.courseId) continue;
    const at = e.term ? termOrdinal(e.term) : null;
    if (target !== null && at !== null && at > target) continue;
    const timing = target !== null && at !== null && at === target ? "concurrent" : "completed";
    credits.push({
      courseId: e.courseId,
      timing,
      source: e.source,
      grade: entryGrade(e),
      term: e.term,
    });
  }
  for (const p of input.planned ?? []) {
    const at = termOrdinal(p.term);
    if (target === null || at === null || at >= target) continue;
    credits.push({ courseId: p.courseId, timing: "planned", source: "plan", term: p.term });
  }
  for (const id of input.sameTerm ?? [])
    credits.push({ courseId: id, timing: "concurrent", source: "schedule", term: input.term });

  return createContext({
    credits,
    equivalents: buildEquivalence(input.catalog, input.equivalentGroups),
    overrides: input.record.overrides,
  });
}

export type ContextState =
  | { status: "loading"; ctx: null; catalog: null }
  | { status: "error"; error: string; ctx: null; catalog: null }
  | { status: "ready"; ctx: RequisiteContext; catalog: CatalogIndex };

/**
 * Cross-listed groups for a term. Sections that meet together aren't always the same course (Intermediate French I
 * and II share a room): only the ones with the same title count for each other.
 */
export function crossListGroups(term: Term): string[][] {
  return term.courses.flatMap((c) => {
    const same = c.crossListed.filter((id) => {
      const other = term.courseById.get(id);
      return other ? sameCourseTitle(c.title, other.title) : false;
    });
    return same.length ? [[c.id, ...same]] : [];
  });
}

/** Course ids in a schedule (visible entries). */
export function scheduleCourseIds(schedule: Schedule | null | undefined): string[] {
  return (schedule?.entries ?? []).filter((e) => !e.hidden).map((e) => e.courseId);
}

/**
 * Courses planned for other terms: the active degree plan's semesters plus the active schedule of every other
 * term. buildRequisiteContext keeps only those before the term being checked.
 */
export function plannedElsewhere(input: {
  termId: string;
  plan: Plan | null | undefined;
  schedules: Record<string, Schedule>;
  /** termId to the active schedule's id. */
  activeSchedule: Record<string, string>;
}): PlannedCourse[] {
  const { termId, plan, schedules, activeSchedule } = input;
  const out: PlannedCourse[] = [];
  const seen = new Set<string>();
  const add = (courseId: string, term: string) => {
    const key = `${courseId}@${term}`;
    if (term === termId || seen.has(key)) return;
    seen.add(key);
    out.push({ courseId, term });
  };
  // Synced semesters count their schedule's courses (resolvePlan), like the degree plan itself.
  for (const t of plan ? resolvePlan(plan, schedules).terms : [])
    for (const c of t.courses) add(c.courseId, t.term);
  for (const [term, id] of Object.entries(activeSchedule)) {
    for (const e of schedules[id]?.entries ?? []) if (!e.hidden) add(e.courseId, term);
  }
  return out;
}

/**
 * Requisite context from the record, courses planned for earlier terms (active degree plan and other terms'
 * active schedules), and the active schedule for `term` itself (same-term, so corequisites only).
 * The degree planner calls buildRequisiteContext() with its own plan instead.
 */
export function requisiteContext(input: {
  term: Term;
  record: RecordData;
  catalog: CatalogIndex;
  plan: Plan | null | undefined;
  schedules: Record<string, Schedule>;
  activeSchedule: Record<string, string>;
}): RequisiteContext {
  const termId = input.term.meta.id;
  const active = input.activeSchedule[termId];
  return buildRequisiteContext({
    record: input.record,
    catalog: input.catalog,
    term: termId,
    sameTerm: scheduleCourseIds(active ? input.schedules[active] : null),
    planned: plannedElsewhere({
      termId,
      plan: input.plan,
      schedules: input.schedules,
      activeSchedule: input.activeSchedule,
    }),
    equivalentGroups: crossListGroups(input.term),
  });
}
