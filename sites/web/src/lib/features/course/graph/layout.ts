// Places the requisite graph (./model.ts): the course in the middle, what it requires growing to its left and what
// it unlocks to its right. Each group is a bracket: its label, a spine, and a tick to each item, whose near edges line
// up. A subtree is measured before it's placed (its height, width and the height its line comes in at), so siblings
// stack without overlapping and every parent sits level with the middle of its bracket. Phones get ./outline.ts.
import {
  knobCenter,
  labelWidth,
  leafWidth,
  type GraphLayout,
  type Item,
  type Line,
  type Metrics,
  type Orientation,
  type Placed,
  type Rect,
  type Side,
} from "./geometry.ts";
import { layoutOutline } from "./outline.ts";
import type { CourseNode, GraphModel, GraphNode, GroupNode, LinkState, Relation } from "./types.ts";

/** Lays out in side coordinates: u runs away from the middle course, v down. */
interface Out {
  box: (item: Item, u1: number, u2: number, v1: number, v2: number) => void;
  line: (
    u1: number,
    v1: number,
    u2: number,
    v2: number,
    state: LinkState | undefined,
    relation: Relation,
  ) => void;
}

interface Sub {
  h: number;
  /** Where its incoming line meets it, from its top. */
  anchor: number;
  w: number;
  /** Plain pills stack closer than subtrees. */
  plain: boolean;
  emit: (u0: number, top: number, out: Out) => void;
}

function measure(node: GraphNode, m: Metrics): Sub {
  if (node.kind === "course") return course(node, m);
  if (node.kind === "group") return group(node, m);
  const w = leafWidth(node, m);
  const item: Item = node.kind === "text" ? { kind: "text", node } : { kind: "more", node };
  return {
    h: m.pillH,
    anchor: m.pillH / 2,
    w,
    plain: true,
    emit: (u0, top, out) => {
      out.box(item, u0, u0 + w, top, top + m.pillH);
    },
  };
}

function course(node: CourseNode, m: Metrics): Sub {
  const requires = node.expanded ? node.requires : null;
  const knobSpace = node.expandable ? m.knob + 6 : 0;
  const pill = (u0: number, top: number, out: Out): void => {
    out.box({ kind: "course", node }, u0, u0 + m.pillW, top, top + m.pillH);
    if (!node.expandable) return;
    const c = knobCenter(u0, m);
    const mid = top + m.pillH / 2;
    out.box(
      { kind: "knob", node },
      c - m.knob / 2,
      c + m.knob / 2,
      mid - m.knob / 2,
      mid + m.knob / 2,
    );
  };
  if (requires === null)
    return { h: m.pillH, anchor: m.pillH / 2, w: m.pillW + knobSpace, plain: true, emit: pill };
  const child = measure(requires, m);
  // The course sits level with the line into what it requires; whichever starts higher sets the top.
  const pillTop = Math.max(0, child.anchor - m.pillH / 2);
  const childTop = Math.max(0, m.pillH / 2 - child.anchor);
  const lead = Math.max(m.lead, knobSpace + 12);
  return {
    h: Math.max(pillTop + m.pillH, childTop + child.h),
    anchor: pillTop + m.pillH / 2,
    w: m.pillW + lead + child.w,
    plain: false,
    emit: (u0, top, out) => {
      const v = top + pillTop + m.pillH / 2;
      out.line(u0 + m.pillW, v, u0 + m.pillW + lead, v, requires.link, requires.relation);
      pill(u0, top + pillTop, out);
      child.emit(u0 + m.pillW + lead, top + childTop, out);
    },
  };
}

/** Stacks subtrees: each one's top, and the total height. */
function stack(subs: Sub[], m: Metrics): { tops: number[]; h: number } {
  const tops: number[] = [];
  let v = 0;
  subs.forEach((sub, i) => {
    const previous = subs[i - 1];
    if (previous !== undefined) v += previous.plain && sub.plain ? m.gap : m.gap * 2;
    tops.push(v);
    v += sub.h;
  });
  return { tops, h: v };
}

function group(node: GroupNode, m: Metrics): Sub {
  const subs = node.items.map((item) => measure(item, m));
  const { tops, h } = stack(subs, m);
  const anchors = subs.map((sub, i) => (tops[i] ?? 0) + sub.anchor);
  const first = anchors[0] ?? 0;
  const last = anchors.at(-1) ?? first;
  const labelW = labelWidth(node, m);
  const spine = labelW === 0 ? 0 : labelW + m.labelGap;
  const itemsAt = spine + m.tick;
  const mid = (first + last) / 2;
  // A label taller than the bracket it sits on (a group of one) pushes the items down.
  const shift = labelW === 0 ? 0 : Math.max(0, m.labelH / 2 - mid);
  return {
    h: Math.max(h + shift, mid + shift + m.labelH / 2),
    anchor: mid + shift,
    w: itemsAt + Math.max(0, ...subs.map((sub) => sub.w)),
    plain: false,
    emit: (u0, top, out) => {
      const v = top + mid + shift;
      if (labelW > 0) {
        out.box({ kind: "label", node }, u0, u0 + labelW, v - m.labelH / 2, v + m.labelH / 2);
        // The line runs under the label, so a label narrower than its slot (a short department) still meets it.
        out.line(u0, v, u0 + spine, v, node.state, node.relation);
      }
      if (last > first)
        out.line(
          u0 + spine,
          top + first + shift,
          u0 + spine,
          top + last + shift,
          node.state,
          node.relation,
        );
      subs.forEach((sub, i) => {
        const item = node.items[i];
        if (item === undefined) return;
        const itemTop = top + (tops[i] ?? 0) + shift;
        const a = itemTop + sub.anchor;
        out.line(u0 + spine, a, u0 + itemsAt, a, item.link, item.relation);
        sub.emit(u0 + itemsAt, itemTop, out);
      });
    },
  };
}

function sideOut(side: Side, edge: number, items: Placed[], lines: Line[]): Out {
  const toX = (u: number): number => (side === "left" ? edge - u : edge + u);
  return {
    box: (item, u1, u2, v1, v2) => {
      items.push({
        ...item,
        side,
        x: Math.min(toX(u1), toX(u2)),
        y: v1,
        w: Math.abs(u2 - u1),
        h: v2 - v1,
      });
    },
    line: (u1, v1, u2, v2, state, relation) => {
      lines.push({ x1: toX(u1), y1: v1, x2: toX(u2), y2: v2, state, relation });
    },
  };
}

/** Places the whole graph, with `m.pad` around it. */
export function layoutGraph(
  model: GraphModel,
  m: Metrics,
  orientation: Orientation = "horizontal",
): GraphLayout {
  if (orientation === "outline") return layoutOutline(model, m);
  const left = model.requires === null ? null : measure(model.requires, m);
  const right = model.unlocks === null ? null : measure(model.unlocks, m);
  const rootX = m.pad + (left === null ? 0 : m.lead + left.w);
  const center = m.pad + Math.max(m.rootH / 2, left?.anchor ?? 0, right?.anchor ?? 0);
  const root: Rect = { x: rootX, y: center - m.rootH / 2, w: m.rootW, h: m.rootH };
  const items: Placed[] = [];
  const lines: Line[] = [];
  let bottom = root.y + root.h;
  if (left !== null && model.requires !== null) {
    const out = sideOut("left", rootX, items, lines);
    out.line(0, center, m.lead, center, model.requires.link, model.requires.relation);
    left.emit(m.lead, center - left.anchor, out);
    bottom = Math.max(bottom, center - left.anchor + left.h);
  }
  if (right !== null) {
    const out = sideOut("right", rootX + m.rootW, items, lines);
    out.line(0, center, m.lead, center, undefined, "unlock");
    right.emit(m.lead, center - right.anchor, out);
    bottom = Math.max(bottom, center - right.anchor + right.h);
  }
  const width = rootX + m.rootW + (right === null ? 0 : m.lead + right.w) + m.pad;
  return { width, height: bottom + m.pad, root, items, lines };
}
