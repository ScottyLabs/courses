import { findNode } from "../audit.ts";
import type { Loadable } from "../catalog.ts";
import type { ProgramVersion } from "../degree-types.ts";
import type { Plan } from "../plan.ts";
import { flattenRequirements } from "../requirement-match.ts";
import { entryYearOf } from "../degree/hooks.ts";
import type { ReqRef } from "./filters.ts";

export interface ProgramChoice {
  auditId: number;
  /** Program name, e.g. "BS in Computer Science". */
  label: string;
  /** Version, e.g. "EY2025 Pittsburgh". */
  detail: string;
}

export interface PlanPrograms {
  hasPlan: boolean;
  choices: ProgramChoice[];
  programIds: number[];
  entryYear: number | null;
  loading: boolean;
}

/**
 * The active degree plan's programs plus the bundles they bring in (gen-eds, concentrations). `versions` is
 * core/programs.ts's loadAuditVersions() for the plan's programs.
 */
export function planPrograms(
  plan: Plan | null,
  versions: Loadable<Map<number, ProgramVersion>>,
): PlanPrograms {
  const programs = plan?.programs ?? [];
  const programIds = programs.map((p) => p.catalogId);
  const entryYear = plan ? entryYearOf(plan.startTerm) : null;
  if (versions.status !== "ready")
    return {
      hasPlan: Boolean(plan),
      choices: [],
      programIds,
      entryYear,
      loading: versions.status === "loading",
    };
  // Selected programs first, then what they attach.
  const selected = programs
    .map((p) => versions.value.get(p.auditId))
    .filter((v): v is ProgramVersion => Boolean(v));
  const rest = [...versions.value.values()].filter((v) => !selected.includes(v));
  const choices = [...selected, ...rest].map((v) => ({
    auditId: v.auditId,
    label: v.programName,
    detail: v.name,
  }));
  return { hasPlan: Boolean(plan), choices, programIds, entryYear, loading: false };
}

export interface RequirementLabel {
  status: "none" | "loading" | "missing" | "ready";
  name?: string;
  program?: string;
  path?: string[];
}

/** Name and program of a selected requirement, for chips and the picker's summary. `versions` holds req.auditId. */
export function requirementLabel(
  req: ReqRef | null,
  versions: Loadable<Map<number, ProgramVersion>>,
): RequirementLabel {
  if (!req) return { status: "none" };
  if (versions.status === "loading") return { status: "loading" };
  const v = versions.status === "ready" ? versions.value.get(req.auditId) : undefined;
  if (!v) return { status: "missing" };
  const flat = flattenRequirements(v).find((f) => f.nodeId === req.nodeId);
  if (flat) return { status: "ready", name: flat.name, program: v.programName, path: flat.path };
  // Wrappers and lone courses aren't in the picker's list; fall back to the raw tree.
  const node = findNode(v.root, req.nodeId);
  return node
    ? { status: "ready", name: node.name, program: v.programName, path: [] }
    : { status: "missing" };
}
