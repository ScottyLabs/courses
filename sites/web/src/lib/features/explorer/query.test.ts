/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../../testing.ts";
import { EMPTY_FILTERS } from "../../core/explorer/filters.ts";
import type { CatalogEntry } from "../../core/catalog-types.ts";
import {
  clearFilters,
  EMPTY_EXTRA,
  extraFilterCount,
  extraMatches,
  parseExplorer,
  writeExplorer,
  type ExplorerQuery,
} from "./query.ts";

const entry = (patch: Partial<CatalogEntry>): CatalogEntry => ({
  id: "15122",
  code: "15-122",
  title: "Principles of Imperative Computation",
  units: 12,
  prereqs: null,
  coreqs: [],
  antireqs: [],
  equivalents: [],
  passFail: false,
  ...patch,
});

describe("explorer query", () => {
  it("round-trips through URL parameters", () => {
    const query: ExplorerQuery = {
      term: "S27",
      q: "algorithms",
      sort: "units",
      chart: true,
      unoffered: true,
      course: "15-122",
      filters: { ...EMPTY_FILTERS, depts: ["CS"], fits: true, req: { auditId: 1, nodeId: 2 } },
      extra: { unitsLow: 9, unitsHigh: 12, number: "3", seasons: ["fall", "summer"] },
    };
    expect(parseExplorer(writeExplorer(query))).toEqual(query);
  });

  it("leaves defaults out of the URL", () => {
    const query = parseExplorer(new URLSearchParams(""));
    expect(writeExplorer(query).toString()).toBe("");
    expect(query.extra).toEqual(EMPTY_EXTRA);
  });

  it("ignores malformed values", () => {
    const query = parseExplorer(new URLSearchParams("ulo=-3&uhi=x&num=12&offered=winter,fall"));
    expect(query.extra).toEqual({ ...EMPTY_EXTRA, seasons: ["fall"] });
  });

  it("orders a reversed units range", () => {
    const query = parseExplorer(new URLSearchParams("ulo=12&uhi=6"));
    expect([query.extra.unitsLow, query.extra.unitsHigh]).toEqual([6, 12]);
  });

  it("clears filters but keeps the search and view", () => {
    const query = parseExplorer(new URLSearchParams("q=x&term=F26&dept=CS&ulo=9&unoffered=1"));
    const cleared = clearFilters(query);
    expect(cleared.filters).toEqual(EMPTY_FILTERS);
    expect(cleared.extra).toEqual(EMPTY_EXTRA);
    expect([cleared.q, cleared.term, cleared.unoffered]).toEqual(["x", "F26", true]);
  });

  it("counts the extra filters", () => {
    expect(extraFilterCount({ unitsLow: 0, unitsHigh: 12, number: "1", seasons: ["fall"] })).toBe(
      3,
    );
    expect(extraFilterCount(EMPTY_EXTRA)).toBe(0);
  });
});

describe("extra filters", () => {
  const range = { ...EMPTY_EXTRA, unitsLow: 9, unitsHigh: 12 };

  it("matches fixed units inside the range", () => {
    expect(extraMatches({ id: "15122", units: 12 }, range)).toBe(true);
    expect(extraMatches({ id: "15122", units: 6 }, range)).toBe(false);
  });

  it("matches variable units whose catalog range overlaps", () => {
    expect(
      extraMatches({ id: "15400", units: null, entry: entry({ units: 3, unitsMax: 9 }) }, range),
    ).toBe(true);
    expect(
      extraMatches({ id: "15400", units: null, entry: entry({ units: 1, unitsMax: 6 }) }, range),
    ).toBe(false);
    expect(extraMatches({ id: "15400", units: null }, range)).toBe(false);
  });

  it("matches the course-number level", () => {
    const level = { ...EMPTY_EXTRA, number: "2" };
    expect(extraMatches({ id: "15213", units: 12 }, level)).toBe(true);
    expect(extraMatches({ id: "15122", units: 12 }, level)).toBe(false);
  });

  it("matches seasons from the catalog or from offered semesters", () => {
    const fall = { ...EMPTY_EXTRA, seasons: ["fall" as const] };
    expect(
      extraMatches({ id: "15122", units: 12, entry: entry({ offered: ["fall"] }) }, fall),
    ).toBe(true);
    expect(extraMatches({ id: "15122", units: 12, terms: ["F26"] }, fall)).toBe(true);
    expect(extraMatches({ id: "15122", units: 12, terms: ["S27"] }, fall)).toBe(false);
  });
});
