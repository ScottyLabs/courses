/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import type { FceRow } from "../../core/fce.ts";
import { expect } from "../../testing.ts";
import {
  DEFAULT_FCE_OPTIONS,
  aggregateFce,
  normalizeFceOptions,
  parseFceOptions,
  responseRate,
  selectFceRows,
} from "./fce-aggregate.ts";

// Made-up people and numbers.
function row(
  term: string,
  courseId: string,
  patch: Partial<FceRow> = {},
): FceRow & { courseId: string } {
  return {
    term,
    courseId,
    section: "1",
    instructor: "EXAMPLE, ADA",
    enrolled: 100,
    responses: 50,
    hours: 10,
    teachingRate: 4,
    courseRate: 4,
    questions: [],
    mini: null,
    ...patch,
  };
}

describe("selectFceRows", () => {
  const rows = [
    row("F24", "15112"),
    row("S25", "15112"),
    row("M25", "15112"),
    row("F25", "15122"),
    row("S26", "15112", { responses: 0 }),
  ];

  it("keeps the most recent semesters with responses in the chosen seasons, newest first", () => {
    expect(selectFceRows(rows, DEFAULT_FCE_OPTIONS).map((r) => r.term)).toEqual(["F25", "S25"]);
  });

  it("counts every semester when the limit is off", () => {
    const options = { semesters: null, seasons: ["S", "M", "F"] as const };
    expect(selectFceRows(rows, options).map((r) => r.term)).toEqual(["F25", "M25", "S25", "F24"]);
  });

  it("limits to the chosen courses", () => {
    expect(selectFceRows(rows, DEFAULT_FCE_OPTIONS, ["15112"]).map((r) => r.term)).toEqual([
      "S25",
      "F24",
    ]);
  });

  it("keeps every row of a kept semester, by course then section", () => {
    const two = [
      row("F25", "15122"),
      row("F25", "15112", { section: "2" }),
      row("F25", "15112"),
      row("S25", "15112"),
    ];
    const kept = selectFceRows(two, { ...DEFAULT_FCE_OPTIONS, semesters: 1 });
    expect(kept.map((r) => `${r.courseId} ${r.section}`)).toEqual([
      "15112 1",
      "15112 2",
      "15122 1",
    ]);
  });
});

describe("aggregateFce", () => {
  it("weights by responses and skips rows without any", () => {
    const result = aggregateFce([
      row("F25", "15112", { responses: 30, hours: 10, teachingRate: 5, courseRate: 3 }),
      row("S25", "15112", { responses: 10, hours: 20, teachingRate: 1, courseRate: null }),
      row("F24", "15112", { responses: 0, hours: 99 }),
    ]);
    expect(result.terms).toEqual(["F25", "S25"]);
    expect(result.responses).toBe(40);
    expect(result.hours).toBeCloseTo(12.5);
    expect(result.teachingRate).toBeCloseTo(4);
    expect(result.courseRate).toBeCloseTo(3);
  });

  it("is empty for no rows", () => {
    expect(aggregateFce([])).toEqual({
      hours: null,
      teachingRate: null,
      courseRate: null,
      responses: 0,
      terms: [],
    });
  });
});

describe("responseRate", () => {
  it("is a percentage of enrollment", () => {
    expect(responseRate({ responses: 25, enrolled: 50 })).toBe(50);
    expect(responseRate({ responses: 0, enrolled: 0 })).toBeNull();
  });
});

describe("parseFceOptions", () => {
  it("falls back to the defaults", () => {
    expect(parseFceOptions(null)).toEqual(DEFAULT_FCE_OPTIONS);
    expect(parseFceOptions("not json")).toEqual(DEFAULT_FCE_OPTIONS);
    expect(normalizeFceOptions({ semesters: -3 })).toEqual(DEFAULT_FCE_OPTIONS);
  });

  it("keeps valid choices and caps the semester count", () => {
    expect(parseFceOptions('{"semesters":500,"seasons":["M","F"]}')).toEqual({
      semesters: 40,
      seasons: ["M", "F"],
    });
    expect(parseFceOptions('{"semesters":null,"seasons":["S"]}')).toEqual({
      semesters: null,
      seasons: ["S"],
    });
  });

  it("reads seasons stored as flags", () => {
    expect(
      normalizeFceOptions({ semesters: 3, seasons: { spring: false, summer: true, fall: true } }),
    ).toEqual({
      semesters: 3,
      seasons: ["M", "F"],
    });
  });
});
