/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../../testing.ts";
import { gridRows } from "./grid.ts";

const cols = (...terms: string[]) => terms.map((term) => ({ term }));

describe("gridRows", () => {
  it("groups semesters into academic years, fall first", () => {
    const { years, next, nextInLastYear } = gridRows("F25", cols("F25", "S26", "F26", "S27"));
    expect(years.map((y) => y.key)).toEqual(["F25", "F26"]);
    expect(years[0]?.slots.map((s) => s.term)).toEqual(["F25", "S26"]);
    expect(years[0]?.summer).toBe("M26");
    expect(years[0]?.hasSummer).toBe(false);
    expect(next).toBe("F27");
    expect(nextInLastYear).toBe(false);
  });

  it("puts a planned summer last in its year", () => {
    const { years } = gridRows("F25", cols("F25", "S26", "M26", "F26"));
    expect(years[0]?.slots.map((s) => s.term)).toEqual(["F25", "S26", "M26"]);
    expect(years[0]?.hasSummer).toBe(true);
  });

  it("offers removed semesters back where they belong", () => {
    const { years } = gridRows("F25", cols("F25", "F26", "S27"));
    expect(years[0]?.slots).toEqual([
      { term: "F25", column: { term: "F25" } },
      { term: "S26", column: null },
    ]);
  });

  it("adds a plan ending in a fall's spring within that year", () => {
    const { years, next, nextInLastYear } = gridRows("F25", cols("F25", "S26", "F26"));
    expect(next).toBe("S27");
    expect(nextInLastYear).toBe(true);
    expect(years[1]?.slots).toEqual([
      { term: "F26", column: { term: "F26" } },
      { term: "S27", column: null },
    ]);
  });

  it("starts an empty plan from its first semester", () => {
    const { years, next } = gridRows("F25", []);
    expect(years).toEqual([]);
    expect(next).toBe("S26");
  });
});
