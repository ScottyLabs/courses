// The plan's numbers for the rail and the phone toolbar: requirements met or planned, the degree's unit total, the
// overall progress and the units the degree leaves out. Courseweave's PlanHeader.
import {
  degreeProgress,
  tallyRequirements,
  type DegreeProgress,
  type RequirementTally,
} from "$lib/core/degree/progress";
import type { PlanPageContext } from "./context";

export interface PlanNumbers {
  /** Requirements across the plan's programs; null until the audit is ready. */
  requirements: RequirementTally | null;
  /** The first program's unit total. */
  needUnits: number | undefined;
  progress: DegreeProgress | null;
  /** Units in the plan the degree leaves out of its total (StuCo and the like); 0 when there are none. */
  outside: number;
}

export function planNumbers(page: Pick<PlanPageContext, "audit" | "derived">): PlanNumbers {
  const audit = page.audit;
  const requirements =
    audit?.programs.reduce((acc, p) => tallyRequirements(p.root, acc), {
      done: 0,
      inProgress: 0,
      planned: 0,
      total: 0,
    }) ?? null;
  const progress = degreeProgress(audit, page.derived.auditInput.courses, page.derived.unitsOf);
  const outside =
    progress && progress.unitTarget !== null
      ? page.derived.totalUnits - (progress.done + progress.inProgress + progress.planned)
      : 0;
  return {
    requirements,
    needUnits: audit?.programs[0]?.totalUnits?.need,
    progress,
    outside: outside > 0.01 ? outside : 0,
  };
}
