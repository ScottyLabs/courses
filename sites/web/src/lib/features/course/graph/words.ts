// What screen readers hear for a course on the requisite graph: its code and title, minimum grade, the student's
// standing, and the group it belongs to with that group's state, since the brackets themselves are only drawn.
import { codeOf } from "../../../core/requisite-format.ts";
import type { Side } from "./geometry.ts";
import type { CourseNode, GroupNode, LinkState, PillStatus } from "./types.ts";

export const STATUS_WORDS: Record<PillStatus, string> = {
  met: "met",
  "in-progress": "in progress",
  planned: "planned",
  missing: "not met",
  waived: "waived",
  "grade-low": "grade too low",
};

export const STATE_WORDS: Record<LinkState, string> = {
  met: "met",
  waived: "waived",
  planned: "met with planned courses",
  missing: "not met",
  unknown: "needs a check",
};

function groupWords(parent: GroupNode | undefined): string {
  if (
    parent === undefined ||
    parent.op === "unlocks" ||
    parent.op === "dept" ||
    parent.op === "fork"
  )
    return "";
  if (parent.op === "with") return ", corequisite: before or in the same semester";
  if (parent.op === "requires") return "";
  const count = parent.items.filter((i) => i.kind !== "more").length;
  const state = parent.state === undefined ? "" : ` (${STATE_WORDS[parent.state]})`;
  return `, ${parent.title.toLowerCase()} ${count}${state}`;
}

/** "15-210, Parallel and Sequential Data Structures and Algorithms, needs C or better, not met, all of 3 (not met)". */
export function courseLabel(
  node: CourseNode,
  side: Side,
  title: string | undefined,
  parent: GroupNode | undefined,
): string {
  const parts = [codeOf(node.id)];
  if (title !== undefined) parts.push(title);
  if (node.minGrade !== undefined) parts.push(`needs ${node.minGrade} or better`);
  if (node.status === "missing") parts.push(STATUS_WORDS.missing);
  else if (node.status !== undefined)
    parts.push(`${STATUS_WORDS[node.status]}: ${node.note ?? ""}`);
  else if (node.ready === true) parts.push("you could take it next");
  if (side === "right" && node.relation === "coreq") parts.push("lists it as a corequisite");
  if (node.cycle) parts.push("circular: it requires a course nearer the middle");
  if (node.shownElsewhere === true)
    parts.push("its prerequisites are shown where it first appears");
  return parts.join(", ") + groupWords(parent);
}
