// What the requisite graph shows, before it's placed (./layout.ts): the course in the middle, the requisite
// expression it requires (expanded further where the student asked), and the courses it unlocks (./unlocks.ts).
// Every course and group carries the student's standing (./status.ts). Pure, so it's tested without a browser.
import type { CatalogIndex, Requisite } from "../../../core/catalog-types.ts";
import { evaluateRequisite, type RequisiteResult } from "../../../core/requisites.ts";
import { courseStatus, linkOf, weakest } from "./status.ts";
import {
  DEFAULT_LIMITS,
  type BuildOptions,
  type CourseNode,
  type GraphModel,
  type GraphNode,
  type GroupNode,
  type MoreNode,
  type Relation,
  type Walk,
} from "./types.ts";
import { unlocksOf } from "./unlocks.ts";

function moreNode(
  group: string,
  hidden: number,
  less: boolean,
  relation: Relation,
  noun: string,
): MoreNode {
  const label = less ? "Show fewer" : `+${hidden} ${hidden === 1 ? noun.replace(/s$/u, "") : noun}`;
  return { kind: "more", key: `${group}+`, group, hidden, label, less, relation };
}

/** A capped list: its first `limit` items and "+N more", or all of them and "Show fewer" once revealed. */
export function cap(
  walk: Walk,
  key: string,
  items: GraphNode[],
  limit: number,
  noun = "more",
): GraphNode[] {
  if (items.length <= limit + 1) return items;
  const relation = items[0]?.relation ?? "prereq";
  const hidden = items.length - limit;
  if (walk.options.revealed.has(key))
    return [...items, moreNode(key, hidden, true, relation, noun)];
  return [...items.slice(0, limit), moreNode(key, hidden, false, relation, noun)];
}

const LABELS = { all: "all of", any: "one of" } as const;
const TITLES = { all: "All of", any: "One of" } as const;

interface Place {
  key: string;
  /** Course ids from the middle course down to here, to stop at cycles. */
  path: readonly string[];
  depth: number;
  relation: Relation;
}

function fromRequisite(
  walk: Walk,
  req: Requisite,
  result: RequisiteResult | null,
  at: Place,
): GraphNode {
  const input = walk.options;
  if (req.type === "course") return courseNode(walk, req, result, at);
  if (req.type === "text")
    return {
      kind: "text",
      key: at.key,
      text: req.text,
      relation: at.relation,
      link: linkOf(result, input),
    };
  // A group of one is just that item.
  const only = req.items.length === 1 ? req.items[0] : undefined;
  if (only !== undefined) return fromRequisite(walk, only, result?.children?.[0] ?? null, at);
  const items = req.items.map((item, i) =>
    fromRequisite(walk, item, result?.children?.[i] ?? null, { ...at, key: `${at.key}.${i}` }),
  );
  const state = linkOf(result, input);
  return {
    kind: "group",
    key: at.key,
    op: req.type,
    label: req.type === "atLeast" ? `${req.n} of` : LABELS[req.type],
    title: req.type === "atLeast" ? `At least ${req.n} of` : TITLES[req.type],
    state,
    link: state,
    relation: at.relation,
    items: cap(walk, at.key, items, walk.limits.group),
  };
}

function hasRequisites(index: CatalogIndex, id: string): boolean {
  const entry = index.courses[id];
  return entry !== undefined && (entry.prereqs !== null || entry.coreqs.some((c) => c !== id));
}

function courseNode(
  walk: Walk,
  req: Extract<Requisite, { type: "course" }>,
  result: RequisiteResult | null,
  at: Place,
): CourseNode {
  const input = walk.options;
  const key = `${at.key}:${req.id}`;
  const cycle = at.path.includes(req.id);
  const expandable = !cycle && hasRequisites(input.index, req.id);
  const auto = at.depth <= (input.autoDepth ?? 0) && !(input.collapsed?.has(key) ?? false);
  const elsewhere = auto && walk.opened.has(req.id);
  const expanded = expandable && (input.expanded.has(key) || (auto && !elsewhere));
  if (expanded) walk.opened.add(req.id);
  const node: CourseNode = {
    kind: "course",
    key,
    id: req.id,
    relation: at.relation,
    depth: at.depth,
    expandable,
    expanded,
    cycle,
    requires: expanded ? requiresOf(walk, req.id, key, [...at.path, req.id], at.depth + 1) : null,
    link: linkOf(result, input),
  };
  if (req.minGrade !== undefined) node.minGrade = req.minGrade;
  if (expandable && !expanded && elsewhere) node.shownElsewhere = true;
  if (result !== null) {
    const { status, note } = courseStatus(result, input.currentTerm);
    if (!(input.unchecked && status === "missing")) {
      node.status = status;
      node.note = note;
    }
  }
  return node;
}

/** The corequisites as an expression: one of them, before or alongside. */
export function coreqExpression(index: CatalogIndex, id: string): Requisite | null {
  const ids = (index.courses[id]?.coreqs ?? []).filter((c) => c !== id);
  const first = ids[0];
  if (first === undefined) return null;
  if (ids.length === 1) return { type: "course", id: first };
  return { type: "any", items: ids.map((c) => ({ type: "course" as const, id: c })) };
}

/** A course's corequisites, drawn as "with" and dashed. */
function coreqsOf(
  walk: Walk,
  id: string,
  key: string,
  path: readonly string[],
  depth: number,
): GroupNode | null {
  const input = walk.options;
  const coreqs = coreqExpression(input.index, id);
  if (coreqs === null) return null;
  const ctx = input.ctx;
  const result = ctx === null ? null : evaluateRequisite(coreqs, ctx, { allowConcurrent: true });
  const inner = fromRequisite(walk, coreqs, result, {
    key: `${key}>c.0`,
    path,
    depth,
    relation: "coreq",
  });
  const state = ctx?.overrides[id]?.coreqs === true ? "waived" : linkOf(result, input);
  return {
    kind: "group",
    key: `${key}>c`,
    op: "with",
    label: "with",
    title:
      coreqs.type === "any"
        ? "One of these, before or in the same semester"
        : "Before or in the same semester",
    state,
    link: state,
    relation: "coreq",
    items: [inner],
  };
}

/** What a course requires: its prerequisites, its corequisites, or both side by side under a fork. */
function requiresOf(
  walk: Walk,
  id: string,
  key: string,
  path: readonly string[],
  depth: number,
): GraphNode | null {
  const input = walk.options;
  const entry = input.index.courses[id];
  if (entry === undefined) return null;
  const ctx = input.ctx;
  const parts: GraphNode[] = [];
  if (entry.prereqs !== null) {
    const result = ctx === null ? null : evaluateRequisite(entry.prereqs, ctx);
    const node = fromRequisite(walk, entry.prereqs, result, {
      key: `${key}>p`,
      path,
      depth,
      relation: "prereq",
    });
    // Waived in the record: its line and bracket show it, the courses under it keep their own standing.
    if (ctx?.overrides[id]?.prereqs === true) {
      node.link = "waived";
      if (node.kind === "group") node.state = "waived";
    }
    parts.push(node);
  }
  const coreqs = coreqsOf(walk, id, key, path, depth);
  if (coreqs !== null) parts.push(coreqs);
  if (parts.length <= 1) return parts[0] ?? null;
  const link = weakest(parts.map((p) => (p.link === "waived" ? "met" : p.link)));
  return {
    kind: "group",
    key: `${key}>f`,
    op: "fork",
    label: "",
    title: "Requires",
    link,
    relation: "prereq",
    items: parts,
  };
}

/** The graph around `options.rootId`. */
export function buildGraph(options: BuildOptions): GraphModel {
  const walk: Walk = {
    options,
    limits: { ...DEFAULT_LIMITS, ...options.limits },
    opened: new Set([options.rootId]),
  };
  const requires = requiresOf(walk, options.rootId, options.rootId, [options.rootId], 1);
  const list = options.unlocks;
  const unlocks = list === null || list.length === 0 ? null : unlocksOf(walk, list, cap);
  return { rootId: options.rootId, requires, unlocks };
}

function* visit(node: GraphNode): Generator<GraphNode> {
  yield node;
  if (node.kind === "group") for (const item of node.items) yield* visit(item);
  else if (node.kind === "course" && node.requires !== null) yield* visit(node.requires);
}

/** Every node, depth first (the middle course's requires, then its unlocks). */
export function* walkGraph(model: GraphModel): Generator<GraphNode> {
  if (model.requires !== null) yield* visit(model.requires);
  if (model.unlocks !== null) yield* visit(model.unlocks);
}

/** Each node's group, by the node's key. */
export function parentsOf(model: GraphModel): Map<string, GroupNode> {
  const parents = new Map<string, GroupNode>();
  for (const node of walkGraph(model))
    if (node.kind === "group") for (const item of node.items) parents.set(item.key, node);
  return parents;
}
