/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../../../testing.ts";
import { createContext } from "../../../core/requisites.ts";
import {
  DESKTOP_METRICS,
  PHONE_METRICS,
  type GraphLayout,
  type Placed,
  type Rect,
} from "./geometry.ts";
import { layoutGraph } from "./layout.ts";
import { buildGraph } from "./model.ts";
import { nextInDirection } from "./navigation.ts";
import { collisions, done, options, placed } from "./test-fixtures.ts";

const ctx = createContext({ credits: [done("15210")] });
const center = (r: Rect): number => r.y + r.h / 2;

/** The state of the line that comes into a placed course (on the left, at its right edge). */
function lineInto(layout: GraphLayout, p: Placed): string | undefined {
  return layout.lines.find((l) => Math.min(l.x1, l.x2) === p.x + p.w && l.y1 === center(p))?.state;
}

describe("layoutGraph: wide screens", () => {
  it("puts requisites left of the course and unlocks right, inside the bounds and without overlaps", () => {
    const layout = layoutGraph(buildGraph(options({ ctx, autoDepth: 9 })), DESKTOP_METRICS);
    for (const item of layout.items.filter((i) => i.kind !== "knob")) {
      if (item.side === "left") expect(item.x + item.w).toBeLessThanOrEqual(layout.root.x);
      else expect(item.x).toBeGreaterThanOrEqual(layout.root.x + layout.root.w);
      expect(item.x).toBeGreaterThanOrEqual(0);
      expect(item.y).toBeGreaterThanOrEqual(0);
      expect(item.x + item.w).toBeLessThanOrEqual(layout.width);
      expect(item.y + item.h).toBeLessThanOrEqual(layout.height);
    }
    expect(collisions(layout)).toEqual([]);
  });

  it("lines a group's items up and centers its parent on the bracket", () => {
    const model = buildGraph(options({ ctx }));
    const layout = layoutGraph(model, DESKTOP_METRICS);
    const [a, b, c] = [placed(layout, "15210"), placed(layout, "21241"), placed(layout, "15251")];
    expect(a.x).toBe(b.x);
    expect(a.y).toBeLessThan(b.y);
    // The OR group's courses sit further out than the AND group's.
    expect(c.x).toBeLessThan(a.x);
    const label = layout.items.find(
      (i) => i.kind === "label" && i.node.key === model.requires?.key,
    );
    expect(label === undefined ? null : center(label)).toBe(center(layout.root));
  });

  it("draws an expanded course level with what it requires", () => {
    const layout = layoutGraph(
      buildGraph(options({ ctx, expanded: new Set(["15451>p.0:15210"]) })),
      DESKTOP_METRICS,
    );
    const [parent, child] = [placed(layout, "15210"), placed(layout, "15150")];
    expect(child.x + child.w).toBeLessThan(parent.x);
    expect(child.y).toBe(parent.y);
    expect(layout.items.some((i) => i.kind === "knob" && i.node.id === "15210")).toBe(true);
  });
});

describe("layoutGraph: lines", () => {
  it("colors the line into each course by its own state", () => {
    const layout = layoutGraph(buildGraph(options({ ctx })), DESKTOP_METRICS);
    expect(lineInto(layout, placed(layout, "15210"))).toBe("met");
    expect(lineInto(layout, placed(layout, "21241"))).toBe("missing");
  });

  it("is just the course when it has no requisites and unlocks nothing", () => {
    const layout = layoutGraph(
      buildGraph(options({ rootId: "21228", unlocks: [] })),
      DESKTOP_METRICS,
    );
    expect(layout.items).toHaveLength(0);
    expect(layout.width).toBe(DESKTOP_METRICS.rootW + DESKTOP_METRICS.pad * 2);
  });
});

describe("layoutGraph: phones", () => {
  it("reads top to bottom: the course, what it requires, then what it unlocks", () => {
    const model = buildGraph(options({ ctx, autoDepth: 9 }));
    const outline = layoutGraph(model, PHONE_METRICS, "outline");
    expect(outline.width).toBeLessThan(layoutGraph(model, PHONE_METRICS).width);
    expect(outline.width).toBeLessThanOrEqual(393);
    const [requires, deeper, unlock] = [
      placed(outline, "15210"),
      placed(outline, "15150"),
      placed(outline, "15456"),
    ];
    expect(requires.y).toBeGreaterThan(outline.root.y + outline.root.h);
    // Each level is indented under the one that needs it, and the unlocks come after.
    expect(deeper.x).toBeGreaterThan(requires.x);
    expect(deeper.y).toBeGreaterThan(requires.y);
    expect(unlock.y).toBeGreaterThan(
      Math.max(...outline.items.filter((i) => i.side === "left").map((i) => i.y)),
    );
    expect(collisions(outline)).toEqual([]);
  });

  it("heads the requisites with their overall state", () => {
    const outline = layoutGraph(buildGraph(options({ ctx })), PHONE_METRICS, "outline");
    const heading = outline.items.find((i) => i.kind === "label" && i.node.op === "requires");
    expect(heading?.kind === "label" ? heading.node.state : null).toBe("missing");
  });
});

const stop = (key: string, x: number, y: number): Rect & { key: string } => ({
  key,
  x,
  y,
  w: 100,
  h: 30,
});
const grid = [
  stop("a", 0, 0),
  stop("b", 0, 40),
  stop("c", 0, 80),
  stop("d", 200, 40),
  stop("e", 200, 200),
];
const at = (key: string): Rect & { key: string } =>
  grid.find((i) => i.key === key) ?? stop(key, 0, 0);

describe("nextInDirection", () => {
  it("moves along a column and across to the nearest level item", () => {
    expect(nextInDirection(grid, at("a"), "down")?.key).toBe("b");
    expect(nextInDirection(grid, at("b"), "up")?.key).toBe("a");
    expect(nextInDirection(grid, at("c"), "right")?.key).toBe("d");
    expect(nextInDirection(grid, at("d"), "left")?.key).toBe("b");
    expect(nextInDirection(grid, at("a"), "left")).toBeNull();
  });
});
