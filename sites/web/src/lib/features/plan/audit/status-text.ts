import type { RequirementResult } from "$lib/core/plan-types";
import { termLabel } from "$lib/core/requisites";

/** When and how a course counted toward a requirement: "Fall 2025", "Spring 2027, planned", "Credit". */
export function assignedStatus(a: RequirementResult["assigned"][number]): string {
  if (a.status === "credit") return "Credit";
  const when = a.term === null ? "" : termLabel(a.term);
  if (a.status === "completed") return when === "" ? "Taken" : when;
  if (a.status === "in-progress") return when === "" ? "In progress" : `${when}, in progress`;
  return when === "" ? "Planned" : `${when}, planned`;
}
