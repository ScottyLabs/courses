/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../../testing.ts";
import { GRADE_OPTIONS, gradeNote, gradeOptionsWith } from "./grades.ts";
import type { CourseInfo } from "./hooks.ts";

const course = (passFail = false): CourseInfo => ({
  id: "15122",
  code: "15-122",
  title: "Example",
  units: 12,
  passFail,
});

describe("gradeNote", () => {
  it("explains grades that don't count toward QPA", () => {
    expect(gradeNote("P", course())).toBe("Not factored into QPA");
    expect(gradeNote("W", course())).toBe("Not factored into QPA");
    expect(gradeNote("A", course())).toBeNull();
    expect(gradeNote(undefined, course())).toBeNull();
  });
  it("flags a letter grade on a pass/fail-only course", () => {
    expect(gradeNote("B", course(true))).toBe("This course is graded pass/fail only.");
  });
});

describe("gradeOptionsWith", () => {
  it("keeps an imported grade the list doesn't have", () => {
    expect(gradeOptionsWith("A")).toBe(GRADE_OPTIONS);
    expect(gradeOptionsWith("")).toBe(GRADE_OPTIONS);
    expect(gradeOptionsWith("AD").at(-1)).toEqual({ value: "AD", label: "AD" });
  });
});
