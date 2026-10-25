// The requisite graph on phones, top to bottom: the course, then what it requires and what it unlocks, each item
// indented under its group's label with the same brackets and colors, so it fits a narrow screen at full size.
import {
  knobCenter,
  labelWidth,
  leafWidth,
  type GraphLayout,
  type Line,
  type Metrics,
  type Placed,
  type Rect,
  type Side,
} from "./geometry.ts";
import type { GraphModel, GraphNode, GroupNode, LinkState, Relation } from "./types.ts";

interface Out {
  items: Placed[];
  lines: Line[];
  right: number;
  side: Side;
  m: Metrics;
}

interface Row {
  /** Where the line from its group meets it. */
  anchor: number;
  bottom: number;
}

function plainRow(node: GraphNode): boolean {
  return node.kind !== "group" && !(node.kind === "course" && node.expanded);
}

/** A fork's parts sit straight under the level that holds it. */
function unfork(node: GraphNode): GraphNode[] {
  return node.kind === "group" && node.op === "fork" ? node.items : [node];
}

/** Items under a row at `x`, `top`: a spine down their left with a tick to each. Returns their bottom. */
function children(
  items: readonly GraphNode[],
  x: number,
  top: number,
  spine: { state: LinkState | undefined; relation: Relation },
  out: Out,
): number {
  const { m } = out;
  const spineX = x + m.indent / 2;
  const at = x + m.indent;
  let y = top + m.gap;
  let last = top;
  items.forEach((child, i) => {
    const previous = items[i - 1];
    if (previous !== undefined && (!plainRow(previous) || !plainRow(child))) y += m.gap;
    const row = place(child, at, y, out);
    out.lines.push({
      x1: spineX,
      y1: row.anchor,
      x2: at,
      y2: row.anchor,
      state: child.link,
      relation: child.relation,
    });
    last = row.anchor;
    y = row.bottom + m.gap;
  });
  if (last > top) out.lines.push({ x1: spineX, y1: top, x2: spineX, y2: last, ...spine });
  return Math.max(top, y - m.gap);
}

function place(node: GraphNode, x: number, y: number, out: Out): Row {
  const { m, side } = out;
  if (node.kind === "group") {
    const w = labelWidth(node, m);
    out.items.push({ kind: "label", node, side, x, y, w, h: m.labelH });
    out.right = Math.max(out.right, x + w);
    const spine = { state: node.state, relation: node.relation };
    const bottom = children(node.items.flatMap(unfork), x, y + m.labelH, spine, out);
    return { anchor: y + m.labelH / 2, bottom };
  }
  if (node.kind === "text" || node.kind === "more") {
    const w = leafWidth(node, m);
    const rect = { side, x, y, w, h: m.pillH };
    out.items.push(
      node.kind === "text" ? { kind: "text", node, ...rect } : { kind: "more", node, ...rect },
    );
    out.right = Math.max(out.right, x + w);
    return { anchor: y + m.pillH / 2, bottom: y + m.pillH };
  }
  out.items.push({ kind: "course", node, side, x, y, w: m.pillW, h: m.pillH });
  let right = x + m.pillW;
  if (node.expandable) {
    const cx = knobCenter(x, m);
    out.items.push({
      kind: "knob",
      node,
      side,
      x: cx - m.knob / 2,
      y: y + (m.pillH - m.knob) / 2,
      w: m.knob,
      h: m.knob,
    });
    right = cx + m.knob / 2;
  }
  out.right = Math.max(out.right, right);
  const requires = node.expanded ? node.requires : null;
  const bottom =
    requires === null
      ? y + m.pillH
      : children(
          unfork(requires),
          x,
          y + m.pillH,
          { state: requires.link, relation: requires.relation },
          out,
        );
  return { anchor: y + m.pillH / 2, bottom };
}

/** "Requires" heads the requisites under the course, as "Unlocks" heads what it unlocks. */
function requiresHeading(requires: GraphNode): GroupNode {
  return {
    kind: "group",
    key: "r",
    op: "requires",
    label: "requires",
    title: "Requires",
    state: requires.link,
    link: requires.link,
    relation: "prereq",
    items: unfork(requires),
  };
}

/** Places the graph as an outline, with `m.pad` around it. */
export function layoutOutline(model: GraphModel, m: Metrics): GraphLayout {
  const root: Rect = { x: m.pad, y: m.pad, w: m.rootW, h: m.rootH };
  const items: Placed[] = [];
  const lines: Line[] = [];
  let right = root.x + root.w;
  const sections: { node: GroupNode; side: Side }[] = [];
  if (model.requires !== null)
    sections.push({ node: requiresHeading(model.requires), side: "left" });
  if (model.unlocks !== null) sections.push({ node: model.unlocks, side: "right" });
  const spine = root.x + m.indent / 2;
  const at = root.x + m.indent;
  let y = root.y + root.h + m.gap * 2;
  let last = root.y + root.h;
  for (const { node, side } of sections) {
    const out: Out = { items, lines, right, side, m };
    const row = place(node, at, y, out);
    right = out.right;
    lines.push({
      x1: spine,
      y1: row.anchor,
      x2: at,
      y2: row.anchor,
      state: node.link,
      relation: node.relation,
    });
    last = row.anchor;
    y = row.bottom + m.gap * 3;
  }
  if (last > root.y + root.h)
    lines.push({ x1: spine, y1: root.y + root.h, x2: spine, y2: last, relation: "unlock" });
  const bottom = sections.length === 0 ? root.y + root.h : y - m.gap * 3;
  return { width: right + m.pad, height: bottom + m.pad, root, items, lines };
}
