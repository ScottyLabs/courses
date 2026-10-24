/* oxlint-disable typescript/no-floating-promises, typescript/no-unsafe-type-assertion -- Courseweave's tests, ported unchanged (decision 14); node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../../testing.ts";
import { haveData, readData } from "../test-data.ts";
import { indexTerm } from "../data.ts";
import type { TermData } from "../types.ts";
import {
  buildInstructorDocs,
  coInstructors,
  scoreInstructor,
  taughtSections,
  weeklyHours,
} from "./instructorData.ts";

// Needs the real S27 semester (see test-data.ts).
const haveTerm = haveData("S27.json");
const term = indexTerm(
  haveTerm
    ? readData<TermData>("S27.json")
    : {
        term: { id: "S27", name: "Spring 2027" } as TermData["term"],
        generatedAt: "",
        courses: [],
      },
);
const docs = buildInstructorDocs(term);
const doc = (name: string) => docs.find((d) => d.inst.name === name)!;

const describeReal = haveTerm ? describe : describe.skip;

describeReal("instructor search", () => {
  it("matches names accent- and case-insensitively", () => {
    expect(scoreInstructor(doc("Iliano Cervesato"), "cerves")).toBeGreaterThan(0);
    expect(scoreInstructor(doc("Iliano Cervesato"), "ILIANO C")).toBeGreaterThan(100);
  });

  it("matches course numbers in any form", () => {
    for (const q of ["15-122", "15122", "15 122", "15-1"])
      expect(scoreInstructor(doc("Iliano Cervesato"), q)).toBeGreaterThan(0);
    expect(scoreInstructor(doc("Iliano Cervesato"), "21-127")).toBe(0);
  });

  it("ranks name matches above course-title matches", () => {
    expect(scoreInstructor(doc("Iliano Cervesato"), "imperative")).toBeGreaterThan(0);
    expect(scoreInstructor(doc("Iliano Cervesato"), "imperative")).toBeLessThan(100);
  });
});

describeReal("teaching load", () => {
  it("lists sections and co-instructors", () => {
    const inst = term.instructorBySlug.get("iliano-cervesato")!;
    const taught = taughtSections(inst, term);
    expect(taught.map((t) => `${t.course.id}/${t.section.name}`)).toEqual(["15122/1", "15122/2"]);
    expect(coInstructors(inst, taught).map((c) => c.name)).toEqual(["Anne Kohlbrenner"]);
  });

  it("counts cross-listed sections at the same time once", () => {
    // 02-510 and 02-710 meet together MW 2:00-3:20.
    const inst = term.instructorBySlug.get("jian-ma")!;
    const taught = taughtSections(inst, term);
    expect(taught.length).toBeGreaterThanOrEqual(2);
    expect(
      weeklyHours(
        taught.map((t) => t.section),
        term,
      ),
    ).toBeCloseTo((80 * 2) / 60);
  });
});
