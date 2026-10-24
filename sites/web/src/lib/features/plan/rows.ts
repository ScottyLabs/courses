// What a planned course's row shows, shared by the cards and table layouts, and where a dragged course lands.
// Courseweave's useCourseRowInfo and drop logic in features/degree/CourseRows.tsx and TermCard.tsx.
import type { CatalogIndex } from "../../core/catalog-types.ts";
import {
  isVariableUnits,
  offering,
  OFFERING_LABEL,
  planCoursePassFail,
  planCourseUnits,
  termPhase,
  type PlanCheck,
  type PlanCourse,
  type UnitsOf,
} from "../../core/plan.ts";
import type { CheckLevel } from "../../core/requisites.ts";

export interface CourseRowInfo {
  units: number;
  /** The catalog says pass/fail only. */
  passFailOnly: boolean;
  /** Counted pass/fail (elected or pass/fail only). */
  passFail: boolean;
  variable: boolean;
  /** Not on the semester's published schedule, or not usually offered that season. */
  offeringWarning: string | null;
  /** Requisite state for a course in the current or a later semester. */
  level: CheckLevel | null;
  /** What the requisite icon says: the problems, or the level's label. */
  statusText: string | null;
}

export const LEVEL_LABEL: Record<CheckLevel, string> = {
  ok: "Requisites met",
  planned: "Requisites met by planned courses",
  unknown: "Check requisites",
  problem: "Requisite problem",
};

export function courseRowInfo(input: {
  course: PlanCourse;
  term: string;
  current: string;
  catalog: CatalogIndex | null;
  unitsOf: UnitsOf;
  check: PlanCheck | undefined;
  scheduled: Set<string> | null | undefined;
}): CourseRowInfo {
  const { course, term, current, catalog, unitsOf, check, scheduled } = input;
  const future = termPhase(term, current) !== "past";
  const o = future ? offering(course.courseId, term, catalog, scheduled) : null;
  const level = future && check ? check.check.level : null;
  const issues = check?.check.issues.filter((i) => i.severity !== "info") ?? [];
  return {
    units: planCourseUnits(course, unitsOf),
    passFailOnly: catalog?.courses[course.courseId]?.passFail === true,
    passFail: planCoursePassFail(course, catalog),
    variable: isVariableUnits(course.courseId, catalog),
    offeringWarning: o === "not-scheduled" || o === "other-seasons" ? OFFERING_LABEL[o] : null,
    level,
    statusText:
      level === null
        ? null
        : issues.length > 0
          ? issues.map((i) => i.message).join("\n")
          : LEVEL_LABEL[level],
  };
}

/**
 * Whether dropping `moved` just before `before` (null: at the end) leaves it where it is: dropped on its own slot,
 * just before itself or the course after it.
 */
export function dropsInPlace(
  planned: { id: string }[],
  moved: string,
  before: string | null,
): boolean {
  const at = planned.findIndex((c) => c.id === moved);
  if (at < 0) return false;
  return before === moved || before === (planned[at + 1]?.id ?? null);
}

/** Where a course one step up or down goes: just before this course (null: at the end), or undefined at an edge. */
export function nudgeTarget(
  planned: { id: string }[],
  index: number,
  direction: "up" | "down",
): string | null | undefined {
  if (direction === "up") return index > 0 ? planned[index - 1]?.id : undefined;
  if (index >= planned.length - 1) return undefined;
  return planned[index + 2]?.id ?? null;
}
