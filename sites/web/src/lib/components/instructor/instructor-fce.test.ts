/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../../testing.ts";
import type { FceRow, InstructorFce } from "../../core/fce.ts";
import { NameIndex } from "../../core/fce-names.ts";
import { loadInstructorFce, recentTeachingRate, type InstructorRow } from "./instructor-fce.ts";

// Made-up people and numbers.
function row(term: string, courseId: string, patch: Partial<FceRow> = {}): InstructorRow {
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

// One made-up instructor with two courses in two FCE documents.
const summary: InstructorFce = {
  name: "Ada Example",
  courses: [
    {
      courseId: "15112",
      terms: 2,
      responses: 80,
      teachingRate: 4,
      courseRate: 4,
      hours: 10,
      last: "F25",
      first: "S25",
    },
    {
      courseId: "21127",
      terms: 1,
      responses: 20,
      teachingRate: 3,
      courseRate: 3,
      hours: 8,
      last: "F24",
      first: "F24",
    },
  ],
  terms: [
    { term: "F24", teachingRate: 3, responses: 20 },
    { term: "S25", teachingRate: 4, responses: 40 },
    { term: "F25", teachingRate: 5, responses: 40 },
  ],
};
const index = new NameIndex<InstructorFce>();
index.add("EXAMPLE, ADA", summary);
const requested: string[] = [];
const source = {
  fceInstructors: () => Promise.resolve(index),
  fceCourses: (prefix: string) => {
    requested.push(prefix);
    const rows: Record<string, FceRow[]> = {
      "15": [
        row("F25", "15112"),
        row("S25", "15112"),
        row("F25", "15112", { instructor: "OTHER, BO" }),
      ],
      "21": [row("F24", "21127")],
    };
    return Promise.resolve(new Map([[prefix === "15" ? "15112" : "21127", rows[prefix] ?? []]]));
  },
};

describe("loadInstructorFce", () => {
  it("loads the rows of every course they have FCEs for, newest first", async () => {
    const data = await loadInstructorFce(source, "Ada Example");
    expect(data?.summary).toBe(summary);
    expect(data?.rows.map((r) => `${r.term} ${r.courseId}`)).toEqual([
      "F25 15112",
      "S25 15112",
      "F24 21127",
    ]);
    expect(requested.toSorted()).toEqual(["15", "21"]);
  });

  it("is null for someone without FCEs", async () => {
    expect(await loadInstructorFce(source, "Nobody Here")).toBeNull();
  });

  it("gives the recent teaching rating", () => {
    expect(recentTeachingRate(summary, 2)).toBeCloseTo(4.5);
  });
});
