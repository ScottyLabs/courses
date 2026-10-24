/* oxlint-disable eslint/max-lines, eslint/no-negated-condition, oxc/no-map-spread, typescript/strict-boolean-expressions, unicorn/no-array-callback-reference, unicorn/no-array-sort, unicorn/no-negated-condition -- Courseweave's code, ported unchanged (decision 14) so fixes carry over between the two codebases */
// Degree plans: named, multi-year course plans audited against program requirements. The edits Courseweave's plan
// store makes, as pure functions; lib/state/plans.ts holds the plans and saves them.
// Completed coursework lives in the record; plans hold what's planned (and, for past terms, what the student put
// there without adding it to the record).
import { compareTerms, defaultTerms, type Plan, type PlanCourse, type PlanTerm } from "../plan.ts";
import type { ProgramSelection } from "../plan-types.ts";
import { termOrdinal } from "../requisites.ts";
import { uid } from "./store.ts";

export type { Plan, PlanCourse, PlanTerm };

export interface SnapshotCourse {
  courseId: string;
  passFail?: boolean;
  units?: number;
}

export interface NewPlanOptions {
  name?: string;
  startTerm?: string;
  programs?: ProgramSelection[];
  /** The semesters (default: eight from `startTerm`), each a term code or a term with its courses. */
  terms?: (string | { term: string; courses: SnapshotCourse[] })[];
}

export const toPlanCourse = (c: SnapshotCourse): PlanCourse => ({
  id: uid(),
  courseId: c.courseId,
  ...(c.passFail ? { passFail: true } : {}),
  ...(c.units !== undefined ? { units: c.units } : {}),
});

export function sortTerms(terms: PlanTerm[]): PlanTerm[] {
  return [...terms].sort((a, b) => compareTerms(a.term, b.term));
}

export function nextPlanName(plans: Plan[]): string {
  let n = plans.length + 1;
  while (plans.some((p) => p.name === `Plan ${n}`)) n++;
  return `Plan ${n}`;
}

function withTerm(terms: PlanTerm[], term: string): PlanTerm[] {
  return terms.some((t) => t.term === term) ? terms : sortTerms([...terms, { term, courses: [] }]);
}

/** Plans, oldest first (tab order). */
export function planList(plans: Iterable<Plan>): Plan[] {
  return [...plans].sort((a, b) => a.createdAt - b.createdAt);
}

/** The most recently edited plan other than `except`: what becomes active when the active plan is deleted. */
export function latestPlan(plans: Iterable<Plan>, except?: string): Plan | undefined {
  return [...plans].filter((p) => p.id !== except).sort((a, b) => b.updatedAt - a.updatedAt)[0];
}

export function newPlan(
  id: string,
  opts: NewPlanOptions,
  existing: Iterable<Plan>,
  now: number,
): Plan {
  const startTerm = (opts.startTerm ?? "F25").toUpperCase();
  return {
    id,
    name: opts.name ?? nextPlanName([...existing]),
    programs: opts.programs ?? [],
    startTerm,
    terms: sortTerms(
      (opts.terms ?? defaultTerms(startTerm)).map((t) =>
        typeof t === "string"
          ? { term: t, courses: [] }
          : { term: t.term, courses: t.courses.map(toPlanCourse) },
      ),
    ),
    createdAt: now,
    updatedAt: now,
  };
}

/** A copy of `p` with new course ids, named "<name> copy". */
export function duplicatePlan(p: Plan, id: string, now: number): Plan {
  return {
    id,
    name: `${p.name} copy`,
    programs: p.programs.map((x) => ({ ...x })),
    startTerm: p.startTerm,
    terms: p.terms.map((t) => ({
      term: t.term,
      courses: t.courses.map((c) => ({ ...c, id: uid() })),
      ...(t.link ? { link: { ...t.link } } : {}),
    })),
    createdAt: now,
    updatedAt: now,
  };
}

/** A plan edit: what to change, given the plan as it is. */
export type PlanEdit = (p: Plan) => Partial<Plan>;

/** The edits Courseweave's plan store makes. Each returns the fields to change. */
export const planEdits = {
  rename:
    (name: string): PlanEdit =>
    () => ({ name: name.trim() || "Untitled plan" }),

  setPrograms:
    (programs: ProgramSelection[]): PlanEdit =>
    () => ({ programs }),

  /** Moves the plan's start; adds default semesters from it and drops empty semesters before it. */
  setStartTerm: (startTerm: string): PlanEdit => {
    const start = startTerm.toUpperCase();
    const from = termOrdinal(start);
    return (p) => {
      if (from === null) return {};
      let terms = p.terms.filter((t) => t.courses.length > 0 || (termOrdinal(t.term) ?? 0) >= from);
      for (const t of defaultTerms(start)) terms = withTerm(terms, t);
      return { startTerm: start, terms };
    };
  },

  addTerm:
    (term: string): PlanEdit =>
    (p) =>
      termOrdinal(term) === null ? {} : { terms: withTerm(p.terms, term.toUpperCase()) },

  /** Removes a semester and the courses planned in it. */
  removeTerm:
    (term: string): PlanEdit =>
    (p) => ({ terms: p.terms.filter((t) => t.term !== term) }),

  /** Adds a course as plan-course `id` (made by the caller, so it can return it). */
  addCourse:
    (
      id: string,
      term: string,
      courseId: string,
      opts: { passFail?: boolean; units?: number } = {},
    ): PlanEdit =>
    (p) => ({
      terms: withTerm(p.terms, term).map((t) =>
        t.term === term
          ? {
              ...t,
              courses: [
                ...t.courses,
                {
                  id,
                  courseId,
                  ...(opts.passFail ? { passFail: true } : {}),
                  ...(opts.units !== undefined ? { units: opts.units } : {}),
                },
              ],
            }
          : t,
      ),
    }),

  removeCourse:
    (planCourseId: string): PlanEdit =>
    (p) => ({
      terms: p.terms.map((t) => ({
        ...t,
        courses: t.courses.filter((c) => c.id !== planCourseId),
      })),
    }),

  /** Moves a course to another semester (created if needed), at `index` or the end. */
  moveCourse:
    (planCourseId: string, toTerm: string, index?: number): PlanEdit =>
    (p) => {
      const course = p.terms.flatMap((t) => t.courses).find((c) => c.id === planCourseId);
      if (!course) return {};
      const without = p.terms.map((t) => ({
        ...t,
        courses: t.courses.filter((c) => c.id !== planCourseId),
      }));
      return {
        terms: withTerm(without, toTerm).map((t) => {
          if (t.term !== toTerm) return t;
          const courses = [...t.courses];
          courses.splice(
            index === undefined ? courses.length : Math.max(0, Math.min(index, courses.length)),
            0,
            course,
          );
          return { ...t, courses };
        }),
      };
    },

  updateCourse:
    (planCourseId: string, patch: Partial<Pick<PlanCourse, "passFail" | "units">>): PlanEdit =>
    (p) => ({
      terms: p.terms.map((t) => ({
        ...t,
        courses: t.courses.map((c) => {
          if (c.id !== planCourseId) return c;
          const next = { ...c, ...patch };
          if (!next.passFail) delete next.passFail;
          if (next.units === undefined) delete next.units;
          return next;
        }),
      })),
    }),

  /** Replaces a semester's courses, e.g. with a schedule from the schedule planner. */
  setTermCourses:
    (term: string, courses: { courseId: string; passFail?: boolean }[]): PlanEdit =>
    (p) => ({
      terms: withTerm(p.terms, term).map((t) =>
        t.term === term
          ? {
              ...t,
              courses: courses.map((c) => ({
                id: uid(),
                courseId: c.courseId,
                ...(c.passFail ? { passFail: true } : {}),
              })),
            }
          : t,
      ),
    }),

  /**
   * Syncs a semester with a schedule: its courses then come from the schedule. `snapshot` (the courses at link
   * time) is kept as the fallback should the schedule be deleted.
   */
  linkTerm:
    (term: string, scheduleId: string, snapshot: SnapshotCourse[]): PlanEdit =>
    (p) => ({
      terms: withTerm(p.terms, term).map((t) =>
        t.term === term ? { ...t, link: { scheduleId }, courses: snapshot.map(toPlanCourse) } : t,
      ),
    }),

  /** Stops syncing; the semester keeps `courses` (normally the schedule's current courses) as planned courses. */
  unlinkTerm:
    (term: string, courses?: SnapshotCourse[]): PlanEdit =>
    (p) => ({
      terms: p.terms.map((t) =>
        t.term === term
          ? { ...t, link: null, ...(courses ? { courses: courses.map(toPlanCourse) } : {}) }
          : t,
      ),
    }),

  /** Units override for a course in a semester (variable-unit courses), creating its snapshot row if needed. */
  setCourseUnits:
    (term: string, courseId: string, units: number | undefined): PlanEdit =>
    (p) => ({
      terms: withTerm(p.terms, term).map((t) => {
        if (t.term !== term) return t;
        const existing = t.courses.some((c) => c.courseId === courseId);
        const courses = existing
          ? t.courses.map((c) => {
              if (c.courseId !== courseId) return c;
              const next = { ...c, units };
              if (units === undefined) delete next.units;
              return next;
            })
          : [...t.courses, toPlanCourse({ courseId, units })];
        return { ...t, courses };
      }),
    }),

  /** Drops a course from a semester's own list by course id (keeps a synced semester's snapshot current). */
  removeCourseFromTerm:
    (term: string, courseId: string): PlanEdit =>
    (p) => ({
      terms: p.terms.map((t) =>
        t.term === term ? { ...t, courses: t.courses.filter((c) => c.courseId !== courseId) } : t,
      ),
    }),

  /** Points synced semesters at a schedule's new id (a draft schedule that got its id from the server). */
  renameLinkedSchedule:
    (from: string, to: string): PlanEdit =>
    (p) =>
      p.terms.some((t) => t.link?.scheduleId === from)
        ? {
            terms: p.terms.map((t) =>
              t.link?.scheduleId === from ? { ...t, link: { scheduleId: to } } : t,
            ),
          }
        : {},
};

/** Applies an edit, stamping `updatedAt`. Returns the same plan when nothing changed. */
export function applyPlanEdit(p: Plan, edit: PlanEdit, now: number): Plan {
  const patch = edit(p);
  if (Object.keys(patch).length === 0) return p;
  return { ...p, ...patch, updatedAt: now };
}

/** v1 to v2: semesters gained an optional `link` to a schedule (none by default). */
export function migratePlans(
  persisted: unknown,
  version: number,
): { plans: Record<string, Plan>; activePlanId: string | null } {
  const state = (persisted ?? {}) as { plans?: Record<string, Plan>; activePlanId?: string | null };
  if (version < 2) {
    for (const p of Object.values(state.plans ?? {}))
      p.terms = (p.terms ?? []).map((t) => ({ ...t, link: t.link ?? null }));
  }
  return { plans: state.plans ?? {}, activePlanId: state.activePlanId ?? null };
}
