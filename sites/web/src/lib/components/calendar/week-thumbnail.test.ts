/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../../testing.ts";
import type { Day } from "../../core/types.ts";
import type { CalendarBlock } from "../../core/week-grid.ts";
import { thumbnailLayout } from "./week-thumbnail.ts";

const block = (day: Day, start: number, end: number, color = 0): CalendarBlock => ({
  key: `${day}-${start}`,
  day,
  start,
  end,
  color,
  label: "15-122",
});

describe("thumbnailLayout", () => {
  it("shows Monday to Friday, 8 AM to 6 PM, when nothing meets", () => {
    expect(thumbnailLayout([])).toEqual({ days: [0, 1, 2, 3, 4], from: 8, to: 18, rects: [] });
  });

  it("adds a weekend day only when something meets on it", () => {
    expect(thumbnailLayout([block(0, 600, 650), block(5, 600, 650)]).days).toEqual([
      0, 1, 2, 3, 4, 5,
    ]);
    expect(thumbnailLayout([block(6, 600, 650)]).days).toEqual([0, 1, 2, 3, 4, 6]);
  });

  it("widens the hours so the first and last meetings have room around them", () => {
    // 8:00 AM needs half an hour above it, so the grid starts at 7; 5:30 to 6:50 PM ends at 8 PM.
    const layout = thumbnailLayout([block(0, 480, 530), block(2, 1050, 1130)]);
    expect([layout.from, layout.to]).toEqual([7, 20]);
    // 9:30 AM to 4 PM fits in the default range.
    const inside = thumbnailLayout([block(0, 570, 620), block(1, 900, 960)]);
    expect([inside.from, inside.to]).toEqual([8, 18]);
  });

  it("places meetings by day and time, side by side when they overlap", () => {
    const { rects } = thumbnailLayout([
      block(0, 540, 600, 3),
      block(0, 570, 630, 4),
      block(2, 780, 840, 5),
    ]);
    expect(rects).toEqual([
      { color: 3, column: 0, lane: 0, lanes: 2, top: 0.1, height: 0.1 },
      { color: 4, column: 0, lane: 1, lanes: 2, top: 0.15, height: 0.1 },
      { color: 5, column: 2, lane: 0, lanes: 1, top: 0.5, height: 0.1 },
    ]);
  });

  it("puts a weekend meeting in its own column after Friday", () => {
    const { rects } = thumbnailLayout([block(6, 600, 660)]);
    expect(rects[0]?.column).toBe(5);
  });
});
