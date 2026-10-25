/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import type { ProgramInfo } from "../../../core/plan-types.ts";
import { expect } from "../../../testing.ts";
import { countKind, programCampus, searchPrograms } from "./programs.ts";

const version = (auditId: number, campus: string | null = "Pittsburgh") => ({
  auditId,
  name: `EY2025 ${campus ?? ""}`.trim(),
  entryYear: 2025,
  campus,
});

const PROGRAMS: ProgramInfo[] = [
  { catalogId: 1, name: "BS in Computer Science", kind: "major", versions: [version(11)] },
  { catalogId: 2, name: "Minor in Computer Science", kind: "minor", versions: [version(21)] },
  { catalogId: 3, name: "BS in Computer Science", kind: "major", versions: [version(31, "Qatar")] },
  {
    catalogId: 4,
    name: "Additional Major in Computer Science",
    kind: "additional-major",
    versions: [version(41)],
  },
  {
    catalogId: 5,
    name: "SCS Computer Science Concentration in AI",
    kind: "requirements",
    versions: [version(51)],
  },
  { catalogId: 6, name: "BS in Mathematical Sciences", kind: "major", versions: [version(61)] },
];

describe("searchPrograms", () => {
  it("needs every word, in any order", () => {
    expect(
      searchPrograms(PROGRAMS, "science computer", "all", new Set()).map((p) => p.catalogId),
    ).toEqual([1, 4, 2, 5, 3]);
    expect(searchPrograms(PROGRAMS, "  ", "all", new Set())).toEqual([]);
  });

  it("ranks names that start with the query first, then Pittsburgh, then majors", () => {
    const ids = searchPrograms(PROGRAMS, "bs in", "all", new Set()).map((p) => p.catalogId);
    expect(ids).toEqual([1, 6, 3]);
  });

  it("filters by kind and leaves out chosen programs", () => {
    expect(
      searchPrograms(PROGRAMS, "computer", "minor", new Set()).map((p) => p.catalogId),
    ).toEqual([2]);
    expect(
      searchPrograms(PROGRAMS, "computer", "other", new Set()).map((p) => p.catalogId),
    ).toEqual([5]);
    expect(
      searchPrograms(PROGRAMS, "computer", "major", new Set([1])).map((p) => p.catalogId),
    ).toEqual([3]);
  });

  it("lists a kind with an empty query, Pittsburgh first, and nothing for all", () => {
    expect(searchPrograms(PROGRAMS, "", "major", new Set()).map((p) => p.catalogId)).toEqual([
      1, 6, 3,
    ]);
    expect(searchPrograms(PROGRAMS, " ", "minor", new Set()).map((p) => p.catalogId)).toEqual([2]);
    expect(searchPrograms(PROGRAMS, "", "major", new Set([6])).map((p) => p.catalogId)).toEqual([
      1, 3,
    ]);
    expect(searchPrograms(PROGRAMS, "", "all", new Set())).toEqual([]);
  });
});

describe("programCampus", () => {
  it("names a single campus other than Pittsburgh", () => {
    expect(programCampus(PROGRAMS[2])).toBe("Qatar");
    expect(programCampus(PROGRAMS[0])).toBe(null);
  });
});

describe("countKind", () => {
  it("counts programs of a kind", () => {
    expect(countKind(PROGRAMS, "major")).toBe(3);
  });
});
