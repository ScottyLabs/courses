/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../../testing.ts";
import type { Plan, RecordEntryLike } from "../plan.ts";
import { planDerived } from "./hooks.ts";

// A made-up student: a spring with a withdrawal next to a course they passed.
const plan: Plan = {
  id: "p",
  name: "Plan 1",
  programs: [],
  startTerm: "F25",
  terms: [
    { term: "F25", courses: [] },
    { term: "S26", courses: [] },
  ],
  createdAt: 0,
  updatedAt: 0,
};

const entry = (id: string, courseId: string, grade: string): RecordEntryLike => ({
  id,
  courseId,
  source: "taken",
  term: "S26",
  grade,
  units: 9,
});

function derive(entries: RecordEntryLike[]) {
  return planDerived({
    plan,
    schedules: {},
    record: { entries, overrides: {} },
    catalog: { courses: {} },
    term: null,
    fceHours: () => null,
    current: "F26",
  });
}

describe("planDerived", () => {
  it("counts no units for a withdrawn course in its semester's totals", () => {
    const derived = derive([entry("a", "76101", "W"), entry("b", "79104", "A")]);
    const spring = derived?.columns.find((c) => c.term === "S26");
    expect(spring?.totals.units).toBe(9);
    expect(spring?.totals.factorable).toBe(9);
    expect(spring?.totals.nonFactorable).toBe(0);
    expect(spring?.recordUnits.get("a")).toBe(0);
    expect(spring?.recordUnits.get("b")).toBe(9);
  });

  it("still counts a course with no grade yet", () => {
    const derived = derive([{ ...entry("c", "15122", ""), grade: undefined }]);
    expect(derived?.columns.find((c) => c.term === "S26")?.totals.units).toBe(9);
  });
});
