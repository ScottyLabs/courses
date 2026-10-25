// The unlocks side of the requisite graph: every course whose requisites mention the course in the middle, as one
// list or, when there are many, grouped by department (the biggest first), each group and the departments capped
// behind "+N more". A course the student could take next (every other requisite met) is marked.
import type { Unlock } from "../../../core/catalog.ts";
import type { CatalogIndex, Requisite } from "../../../core/catalog-types.ts";
import {
  evaluateRequisite,
  type CourseCredit,
  type RequisiteContext,
} from "../../../core/requisites.ts";
import { courseStatus } from "./status.ts";
import type { CourseNode, GraphNode, GroupNode, Walk } from "./types.ts";

/** Caps a list (./model.ts's `cap`, passed in so the two modules don't import each other). */
export type Cap = (
  walk: Walk,
  key: string,
  items: GraphNode[],
  limit: number,
  noun?: string,
) => GraphNode[];

/** A copy of the context with `id` counted as done, to see what the student could take after it. */
export function withCourseDone(ctx: RequisiteContext, id: string): RequisiteContext {
  const credits = new Map(ctx.credits);
  const done: CourseCredit = { courseId: id, timing: "completed", source: "taken" };
  credits.set(id, [...(credits.get(id) ?? []), done]);
  return { ...ctx, credits };
}

function coreqsOf(index: CatalogIndex, id: string): Requisite | null {
  const ids = (index.courses[id]?.coreqs ?? []).filter((c) => c !== id);
  return ids.length === 0
    ? null
    : { type: "any", items: ids.map((c) => ({ type: "course" as const, id: c })) };
}

/**
 * Whether the student could take `id` once the middle course is done (`done` counts it): they don't have it yet and
 * every other prerequisite is met, its corequisites met or planned, or waived.
 */
export function couldTakeNext(
  index: CatalogIndex,
  id: string,
  ctx: RequisiteContext,
  done: RequisiteContext,
): boolean {
  const entry = index.courses[id];
  if (entry === undefined) return false;
  const own = evaluateRequisite({ type: "course", id }, ctx, { allowConcurrent: true }).status;
  if (own === "met" || own === "planned") return false;
  const override = ctx.overrides[id];
  if (
    entry.prereqs !== null &&
    override?.prereqs !== true &&
    evaluateRequisite(entry.prereqs, done).status !== "met"
  )
    return false;
  const coreqs = coreqsOf(index, id);
  if (coreqs === null || override?.coreqs === true) return true;
  const status = evaluateRequisite(coreqs, done, { allowConcurrent: true }).status;
  return status === "met" || status === "planned";
}

function unlockNode(walk: Walk, u: Unlock, key: string, done: RequisiteContext | null): CourseNode {
  const input = walk.options;
  const node: CourseNode = {
    kind: "course",
    key: `${key}:${u.id}`,
    id: u.id,
    relation: u.via === "coreq" ? "coreq" : "unlock",
    depth: 1,
    expandable: false,
    expanded: false,
    cycle: false,
    requires: null,
    required: u.required,
  };
  const ctx = input.ctx;
  if (ctx === null) return node;
  // An unlock's own status is whether the student has it; "missing" there would only mean "not taken yet".
  const own = evaluateRequisite({ type: "course", id: u.id }, ctx, { allowConcurrent: true });
  if (own.status === "met" || own.status === "planned") {
    const { status, note } = courseStatus(own, input.currentTerm);
    node.status = status;
    node.note = note;
  } else if (!input.unchecked && done !== null && couldTakeNext(input.index, u.id, ctx, done)) {
    node.ready = true;
  }
  return node;
}

const courses = (n: number): string => (n === 1 ? "1 course" : `${n} courses`);

function byDepartment(
  walk: Walk,
  list: readonly Unlock[],
  key: string,
  done: RequisiteContext | null,
  cap: Cap,
): GroupNode[] {
  const byPrefix = new Map<string, Unlock[]>();
  for (const u of list) {
    const prefix = u.id.slice(0, 2);
    byPrefix.set(prefix, [...(byPrefix.get(prefix) ?? []), u]);
  }
  return [...byPrefix]
    .toSorted((a, b) => b[1].length - a[1].length || a[0].localeCompare(b[0]))
    .map(([prefix, us]): GroupNode => {
      const groupKey = `${key}.${prefix}`;
      const name = walk.options.departmentOf(prefix);
      const items = us.map((u): GraphNode => unlockNode(walk, u, groupKey, done));
      return {
        kind: "group",
        key: groupKey,
        op: "dept",
        label: name,
        count: us.length,
        title: `${name}, ${courses(us.length)}`,
        relation: "unlock",
        items: cap(walk, groupKey, items, walk.limits.department),
      };
    });
}

/** What the middle course unlocks: one list, or by department when there are many. */
export function unlocksOf(walk: Walk, list: readonly Unlock[], cap: Cap): GroupNode {
  const ctx = walk.options.ctx;
  const done = ctx === null ? null : withCourseDone(ctx, walk.options.rootId);
  const key = "u";
  const base = {
    kind: "group" as const,
    key,
    op: "unlocks" as const,
    label: "unlocks",
    count: list.length,
    title: `Unlocks ${courses(list.length)}`,
    relation: "unlock" as const,
  };
  const prefixes = new Set(list.map((u) => u.id.slice(0, 2)));
  if (list.length <= walk.limits.flat || prefixes.size === 1) {
    const items = list.map((u): GraphNode => unlockNode(walk, u, key, done));
    return { ...base, items: cap(walk, key, items, walk.limits.list) };
  }
  const groups = byDepartment(walk, list, key, done, cap);
  return { ...base, items: cap(walk, key, groups, walk.limits.departments, "more departments") };
}
