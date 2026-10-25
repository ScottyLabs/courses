// The student's standing in the requisite graph, read off the course pages' own checks: a course's status from its
// `evaluateRequisite()` result, the lines' states, and the middle course's verdict from `checkCourse()` and
// `summarizeCheck()`, so the graph never says something the course pages don't.
import type { CatalogEntry } from "../../../core/catalog-types.ts";
import { plannedConditions, summarizeCheck } from "../../../core/record/check-summary.ts";
import { codeOf } from "../../../core/requisite-format.ts";
import {
  checkCourse,
  missingParts,
  normalizeGrade,
  termLabel,
  type CheckLevel,
  type CourseCheck,
  type CourseCredit,
  type RequisiteContext,
  type RequisiteResult,
} from "../../../core/requisites.ts";
import type { LinkState, PillStatus, StatusInput } from "./types.ts";

const has = (text: string | undefined): text is string => text !== undefined && text !== "";

function inProgress(credit: CourseCredit, currentTerm: string): boolean {
  if (credit.source !== "taken") return false;
  if (credit.timing === "concurrent") return true;
  return credit.term === currentTerm && normalizeGrade(credit.grade) === undefined;
}

function metNote(credit: CourseCredit): string {
  if (credit.source === "transfer") return "Transfer credit";
  if (credit.source === "exam") return "Exam credit";
  const when = has(credit.term) ? `Taken in ${termLabel(credit.term)}` : "Taken";
  const grade = normalizeGrade(credit.grade);
  return grade === undefined ? when : `${when}, ${grade}`;
}

function plannedNote(credit: CourseCredit): string {
  if (credit.timing === "concurrent") return "In this semester's schedule";
  const when = has(credit.term) ? termLabel(credit.term) : null;
  if (credit.source === "schedule")
    return when === null ? "In a schedule" : `In your ${when} schedule`;
  return when === null ? "Planned" : `Planned for ${when}`;
}

/** How a course stands, from its evaluation result, with what it rests on. */
export function courseStatus(
  result: RequisiteResult,
  currentTerm: string,
): { status: PillStatus; note: string } {
  const match = result.match;
  if ((result.status === "met" || result.status === "planned") && match !== undefined) {
    const credit = match.credit;
    const via = match.via === undefined ? "" : ` (as ${codeOf(match.via)})`;
    if (credit.source === "waiver") return { status: "waived", note: `Placed out${via}` };
    if (inProgress(credit, currentTerm)) {
      const when = has(credit.term) ? ` in ${termLabel(credit.term)}` : " this semester";
      return { status: "in-progress", note: `Taking it${when}${via}` };
    }
    if (result.status === "planned") return { status: "planned", note: plannedNote(credit) + via };
    return { status: "met", note: metNote(credit) + via };
  }
  const low = result.gradeTooLow;
  if (low !== undefined)
    return {
      status: "grade-low",
      note: `You have ${low.grade}; it needs ${low.minGrade} or better`,
    };
  return { status: "missing", note: "Not taken" };
}

/** The line into a node, from its evaluation; nothing while unchecked and missing. */
export function linkOf(result: RequisiteResult | null, input: StatusInput): LinkState | undefined {
  if (result === null) return undefined;
  if (input.unchecked && (result.status === "missing" || result.status === "unknown"))
    return undefined;
  return result.status;
}

/** The weakest of several states: what a line joining them shows. */
export function weakest(states: (LinkState | undefined)[]): LinkState | undefined {
  if (states.includes(undefined)) return undefined;
  const order: LinkState[] = ["missing", "unknown", "planned", "met", "waived"];
  return order.find((s) => states.includes(s));
}

// ---------------------------------------------------------------------------
// The middle course's result

export interface GraphSummary {
  check: CourseCheck;
  level: CheckLevel | "none";
  /** The course pages' requisite line (summarizeCheck), word for word. */
  text: string;
  /** The same, briefly, for the course in the middle: "Prerequisites met", "1 missing", "Met once 21-127 is done". */
  short: string;
}

function joinAnd(parts: string[]): string {
  if (parts.length <= 1) return parts.join("");
  return `${parts.slice(0, -1).join(", ")} and ${parts.at(-1) ?? ""}`;
}

function missingCount(check: CourseCheck): number {
  const { prereqs, coreqs } = check;
  const prereq =
    prereqs !== null && !check.waived.prereqs && prereqs.status === "missing"
      ? Math.max(1, missingParts(prereqs).length)
      : 0;
  const coreq = coreqs !== null && !check.waived.coreqs && coreqs.status === "missing" ? 1 : 0;
  return prereq + coreq;
}

function shortText(
  check: CourseCheck,
  summary: { level: CheckLevel | "none"; text: string },
): string {
  if (summary.level === "problem") {
    const missing = missingCount(check);
    return missing > 0 ? `${missing} missing` : summary.text;
  }
  if (summary.level === "planned" && summary.text.startsWith("Met if you pass")) {
    const codes = plannedConditions(check.prereqs).map((c) => codeOf(c.via ?? c.courseId));
    if (codes.length > 0)
      return `Met once ${joinAnd(codes)} ${codes.length === 1 ? "is" : "are"} done`;
  }
  if (summary.level === "unknown") return "Check the conditions";
  return summary.text;
}

/** The middle course's requisite check, as the course pages' status line words it and briefly. */
export function graphSummary(
  courseId: string,
  entry: CatalogEntry | undefined,
  ctx: RequisiteContext,
): GraphSummary {
  const check = checkCourse(courseId, entry, ctx);
  const summary = summarizeCheck(check, entry);
  return { check, ...summary, short: shortText(check, summary) };
}
