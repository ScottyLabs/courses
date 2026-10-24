/* oxlint-disable eslint/max-lines-per-function, oxc/no-map-spread, typescript/strict-boolean-expressions -- Courseweave's code, ported unchanged (decision 14) so fixes carry over between the two codebases */
import type { CatalogIndex } from "../catalog-types.ts";
import type { Term } from "../data.ts";
import { examById, examLabel } from "../exam-credit.ts";
import type { FceSummaryData } from "../fce.ts";
import { nearestTerm, type Offerings } from "../offerings.ts";
import {
  academicYearStart,
  checkPlan,
  currentTermCode,
  defaultTerms,
  incomingCredit,
  isVariableUnits,
  planColumns,
  planCourseUnits,
  planCoursePassFail,
  planToAuditInput,
  resolvePlan,
  effectiveTermCourses,
  termTotals,
  type EffectiveTerm,
  type Plan,
  type PlanCheck,
  type PlanColumn,
  type RecordEntryLike,
  type RecordLike,
  type TermTotals,
  type UnitsOf,
} from "../plan.ts";
import type { AuditInput } from "../plan-types.ts";
import { isFactorableGrade, normalizeGrade } from "../requisites.ts";
import { coursePath } from "../routes.ts";
import type { Schedule } from "../state/store.ts";
import type { TermIndexEntry } from "../types.ts";

/** The term in progress today. */
export function currentTerm(): string {
  return currentTermCode();
}

/** FCE hours/week for a course, or null without data. */
export function fceHours(
  fce: FceSummaryData | null | undefined,
): (courseId: string) => number | null {
  return (id: string) => fce?.courses.get(id)?.hours ?? null;
}

/** Term ids that have published schedule data (and so a schedule planner). */
export function publishedTerms(terms: TermIndexEntry[] | null | undefined): Set<string> {
  return new Set((terms ?? []).map((t) => t.id));
}

/**
 * For plan terms that have published schedule data: term to the set of offered course ids. Terms that fail to load
 * are left out.
 */
export async function scheduledCourses(
  termIds: string[],
  published: Set<string>,
  loadTerm: (id: string) => Promise<Term>,
): Promise<Map<string, Set<string>>> {
  const out = new Map<string, Set<string>>();
  const loaded = await Promise.all(
    termIds.filter((t) => published.has(t)).map((id) => loadTerm(id).catch(() => null)),
  );
  for (const t of loaded) if (t) out.set(t.meta.id, new Set(t.courseById.keys()));
  return out;
}

export interface ColumnView extends PlanColumn {
  totals: TermTotals;
  /** Units/grade facts for the read-only record rows. */
  recordUnits: Map<string, number>;
}

export interface PlanDerived {
  /** The plan with synced semesters' courses taken from their schedules: what every view and check uses. */
  resolved: Plan;
  /** Per semester: its effective courses and, when synced, the schedule (or that the schedule is gone). */
  terms: Map<string, EffectiveTerm>;
  catalog: CatalogIndex | null;
  current: string;
  columns: ColumnView[];
  incoming: RecordEntryLike[];
  checks: Map<string, PlanCheck>;
  auditInput: AuditInput;
  unitsOf: UnitsOf;
  /** Units earned on the record plus everything planned. */
  totalUnits: number;
  completedUnits: number;
}

function examSource(examId: string, score: string): string {
  const exam = examById(examId);
  return exam ? `${examLabel(exam)} (${score})` : `Exam credit (${score})`;
}

export interface PlanDerivedInput {
  plan: Plan | null;
  /** Every schedule, by id: synced semesters read theirs. */
  schedules: Record<string, Schedule>;
  record: RecordLike;
  catalog: CatalogIndex | null;
  /** The semester being browsed, for units the catalog lacks. */
  term: Term | null;
  /** FCE hours per course (fceHours()). */
  fceHours: (courseId: string) => number | null;
  /** The term in progress (currentTerm()). */
  current: string;
}

/** Everything the plan page shows, derived from the plan, the record, the catalog and FCEs. */
export function planDerived(input: PlanDerivedInput): PlanDerived | null {
  const { plan, schedules, record, catalog, term, current } = input;
  if (!plan) return null;
  const unitsOf: UnitsOf = (id) =>
    catalog?.courses[id]?.units ?? term?.courseById.get(id)?.units ?? null;
  const terms = effectiveTermCourses(plan, schedules);
  const resolved = resolvePlan(plan, schedules);
  const columns: ColumnView[] = planColumns(resolved, record, current).map((col) => {
    const recordUnits = new Map<string, number>();
    const items = [
      ...col.recorded.map((e) => {
        const units = e.units ?? unitsOf(e.courseId ?? "") ?? 0;
        recordUnits.set(e.id, units);
        const grade = normalizeGrade(e.grade);
        return {
          courseId: e.courseId ?? "",
          units,
          passFail:
            isFactorableGrade(grade) === false ||
            !!(e.courseId && catalog?.courses[e.courseId]?.passFail),
        };
      }),
      ...col.planned.map((c) => ({
        courseId: c.courseId,
        units: planCourseUnits(c, unitsOf),
        passFail: planCoursePassFail(c, catalog),
        variable: c.units === undefined && isVariableUnits(c.courseId, catalog),
      })),
    ];
    return { ...col, totals: termTotals(items, input.fceHours), recordUnits };
  });
  const auditInput = planToAuditInput(resolved, record, catalog, {
    current,
    unitsOf,
    examLabel: examSource,
  });
  const checks = checkPlan(resolved, record, catalog);
  // The audit input already includes general transfer credit (as credit: rows), so it isn't added again.
  const completedUnits = auditInput.courses
    .filter((c) => c.status === "completed" || c.status === "credit")
    .reduce((n, c) => n + c.units, 0);
  return {
    resolved,
    terms,
    catalog,
    current,
    columns,
    incoming: incomingCredit(record),
    checks,
    auditInput,
    unitsOf,
    totalUnits: auditInput.courses.reduce((n, c) => n + c.units, 0),
    completedUnits,
  };
}

/** Fall and spring terms from six years back to next fall, for start-term pickers. */
export function startTermOptions(current: string): string[] {
  const fall = academicYearStart(current);
  const year = Number(fall.slice(1));
  const from = `F${String(year - 6).padStart(2, "0")}`;
  return defaultTerms(from, 16);
}

export function entryYearOf(startTerm: string): number {
  return 2000 + Number(academicYearStart(startTerm).slice(1));
}

/** Data transfer type for dragging plan courses between semesters. */
export const DRAG_TYPE = "application/x-courseweave-plan-course";
/** A course dragged from the requirements panel onto a semester (adds it). */
export const DRAG_NEW_COURSE = "application/x-courseweave-course";

/**
 * Where a course links from the degree planner: its course page in the semester it's planned for when that semester
 * offers it, else in the semester you're browsing, else in the nearest semester that offers it. A course no semester
 * here offers opens its catalog page.
 */
export function courseHref(
  routed: Term,
  offerings: Offerings | null | undefined,
): (courseId: string, term?: string | null) => string {
  return (id, term) => {
    const terms = offerings?.byId.get(id)?.terms ?? [];
    if (term && terms.includes(term)) return coursePath(id, term);
    if (routed.courseById.has(id)) return coursePath(id, routed.meta.id);
    const nearest = nearestTerm(terms, term ?? routed.meta.id);
    return coursePath(id, nearest ?? null);
  };
}
