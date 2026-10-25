// A small catalog and helpers for the requisite graph's tests. Only tests import this module.
import { buildUnlocks } from "../../../core/catalog.ts";
import type { CatalogEntry, CatalogIndex, Requisite } from "../../../core/catalog-types.ts";
import type { CourseCredit } from "../../../core/requisites.ts";
import type { Placed, Rect, GraphLayout } from "./geometry.ts";
import { walkGraph } from "./model.ts";
import type { BuildOptions, CourseNode, GraphModel, GraphNode, GroupNode } from "./types.ts";

export const course = (id: string, minGrade?: string): Requisite =>
  minGrade === undefined ? { type: "course", id } : { type: "course", id, minGrade };
export const all = (...items: Requisite[]): Requisite => ({ type: "all", items });
export const any = (...items: Requisite[]): Requisite => ({ type: "any", items });

export const entry = (
  id: string,
  prereqs: Requisite | null = null,
  coreqs: string[] = [],
): CatalogEntry => ({
  id,
  code: `${id.slice(0, 2)}-${id.slice(2)}`,
  title: id,
  units: 12,
  prereqs,
  coreqs,
  antireqs: [],
  equivalents: [],
  passFail: false,
});

export function catalog(...entries: CatalogEntry[]): CatalogIndex {
  return { courses: Object.fromEntries(entries.map((e) => [e.id, e])) };
}

// 15-451 needs 15-210, 21-241 and one of 15-251 or 21-228; 15-210 needs 15-150, which needs 15-122 (with a
// corequisite), and 15-251 needs 15-122 too.
export const INDEX = catalog(
  entry(
    "15451",
    all(
      course("15210", "C"),
      course("21241", "C"),
      any(course("15251", "C"), course("21228", "C")),
    ),
  ),
  entry("15210", course("15150", "C")),
  entry("15150", all(course("15122"), any(course("21127"), course("15151")))),
  entry("15122", any(course("15112")), ["21127"]),
  entry("15251", course("15122")),
  entry("21228", null),
  entry("21241", null),
  entry("21127", null),
  entry("15151", null),
  entry("15112", null),
  entry("15456", course("15451")),
  entry("15457", all(course("15451"), course("21241"))),
);

export const done = (courseId: string, grade?: string, term = "F25"): CourseCredit =>
  grade === undefined
    ? { courseId, timing: "completed", source: "taken", term }
    : { courseId, timing: "completed", source: "taken", term, grade };

export const planned = (courseId: string, term = "S27"): CourseCredit => ({
  courseId,
  timing: "planned",
  source: "plan",
  term,
});

export function options(overrides: Partial<BuildOptions> = {}): BuildOptions {
  return {
    index: INDEX,
    rootId: "15451",
    ctx: null,
    unchecked: false,
    currentTerm: "F26",
    expanded: new Set(),
    revealed: new Set(),
    unlocks: buildUnlocks(INDEX).get("15451") ?? [],
    departmentOf: (prefix) => `Dept ${prefix}`,
    ...overrides,
  };
}

export function courses(model: GraphModel): CourseNode[] {
  return [...walkGraph(model)].filter((n): n is CourseNode => n.kind === "course");
}

export function find(model: GraphModel, id: string): CourseNode {
  const node = courses(model).find((n) => n.id === id);
  if (node === undefined) throw new Error(`no ${id}`);
  return node;
}

export function group(node: GraphNode | null | undefined): GroupNode {
  if (node?.kind !== "group") throw new Error("not a group");
  return node;
}

export function placed(layout: GraphLayout, id: string): Placed {
  const item = layout.items.find((i) => i.kind === "course" && i.node.id === id);
  if (item === undefined) throw new Error(`${id} isn't placed`);
  return item;
}

export const overlaps = (a: Rect, b: Rect): boolean =>
  a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;

/** Pairs of drawn boxes (not expand buttons) that overlap, by key, to show in a failed expectation. */
export function collisions(layout: GraphLayout): string[] {
  const boxes = layout.items.filter((i) => i.kind !== "knob");
  return boxes.flatMap((a, i) =>
    boxes
      .slice(i + 1)
      .filter((b) => overlaps(a, b))
      .map((b) => `${a.kind} ${a.node.key} / ${b.kind} ${b.node.key}`),
  );
}
