// Sizes and shapes the requisite graph's layouts (./layout.ts, ./outline.ts) work in and produce.
import type { CourseNode, GroupNode, LinkState, MoreNode, Relation, TextNode } from "./types.ts";

export type Orientation = "horizontal" | "outline";

export interface Metrics {
  pillW: number;
  pillH: number;
  rootW: number;
  rootH: number;
  /** The line from a course to what it requires. */
  lead: number;
  /** From a spine to its items. */
  tick: number;
  /** Between a group's label and its spine. */
  labelGap: number;
  labelH: number;
  /** Between stacked items; twice this around subtrees. */
  gap: number;
  /** The expand button beside a course that has requisites of its own. */
  knob: number;
  /** Department labels on the unlocks side. */
  deptW: number;
  /** A condition that isn't a course, at most. */
  textW: number;
  /** Indent of each outline level (phones). */
  indent: number;
  pad: number;
  /** Width of a bracket label's text. */
  measure: (text: string) => number;
}

/** Approximate width of 12px semibold text. */
export const approximateMeasure = (text: string): number => Math.ceil(text.length * 7.6);

export const DESKTOP_METRICS: Metrics = {
  pillW: 124,
  pillH: 34,
  rootW: 212,
  rootH: 88,
  lead: 32,
  tick: 14,
  labelGap: 10,
  labelH: 22,
  gap: 8,
  knob: 20,
  deptW: 168,
  textW: 200,
  indent: 22,
  pad: 24,
  measure: approximateMeasure,
};

export const PHONE_METRICS: Metrics = {
  ...DESKTOP_METRICS,
  pillW: 124,
  pillH: 40,
  rootW: 248,
  rootH: 92,
  gap: 6,
  indent: 24,
  pad: 12,
};

export type Side = "left" | "right";

export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** Something drawn on the graph besides its lines, before it's placed. */
export type Item =
  | { kind: "course"; node: CourseNode }
  | { kind: "more"; node: MoreNode }
  | { kind: "text"; node: TextNode }
  | { kind: "label"; node: GroupNode }
  | { kind: "knob"; node: CourseNode };

/** An item placed: requisites on the left, unlocks on the right (above and below on phones). */
export type Placed = Item & Rect & { side: Side };

export interface Line {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  state?: LinkState;
  relation: Relation;
}

export interface GraphLayout {
  width: number;
  height: number;
  root: Rect;
  items: Placed[];
  lines: Line[];
}

/** A bracket label's width: its text, the state icon of a requirement, padding; departments get a fixed slot. */
export function labelWidth(node: GroupNode, m: Metrics): number {
  if (node.label === "") return 0;
  if (node.op === "dept") return m.deptW;
  const text = node.count === undefined ? node.label : `${node.label} ${node.count}`;
  return m.measure(text.toUpperCase()) + 16 + (node.state === undefined ? 0 : 15);
}

/** The width of a box that isn't a course (a "+N more" button or a condition). */
export function leafWidth(item: MoreNode | TextNode, m: Metrics): number {
  return item.kind === "text" ? Math.min(m.textW, m.measure(item.text) + 20) : m.pillW;
}

/** The center of the expand button beside a course that starts at `start`. */
export function knobCenter(start: number, m: Metrics): number {
  return start + m.pillW + 3 + m.knob / 2;
}
