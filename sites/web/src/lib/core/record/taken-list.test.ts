/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../../testing.ts";
import type { RecordEntry } from "../state/record.ts";
import {
  ALL_FILTERS,
  courseLevel,
  currentlyTaking,
  departmentKey,
  departmentLabel,
  departmentOptions,
  groupByTerm,
  levelOptions,
  matchesFilters,
  missingFromRecord,
  NO_TERM,
  semesterOptions,
  takenEntries,
  type FactsOf,
} from "./taken-list.ts";

let next = 0;
function entry(patch: Partial<RecordEntry>): RecordEntry {
  next += 1;
  return { id: `e${next}`, courseId: "15122", source: "taken", addedAt: 0, ...patch };
}

// Made-up department facts, as a semester or the offerings document would give them.
const FACTS: Record<string, { dept: string; level: "U" | "G" }> = {
  "15122": { dept: "CS", level: "U" },
  "15619": { dept: "CS", level: "G" },
  "21127": { dept: "MSC", level: "U" },
};
const factsOf: FactsOf = (id) => FACTS[id] ?? {};
const label = (term: string): string => `term ${term}`;

describe("courseLevel and departmentKey", () => {
  it("use the schedule's facts when known", () => {
    expect(courseLevel("15619", { level: "U" })).toBe("U");
    expect(departmentKey("15122", { dept: "CS" })).toBe("CS");
  });
  it("fall back to the course number", () => {
    expect(courseLevel("15619")).toBe("G");
    expect(courseLevel("15122")).toBe("U");
    expect(departmentKey("98242")).toBe("98");
    expect(departmentLabel("98")).toBe("Courses numbered 98-xxx");
    expect(departmentLabel("CS")).toBe("Computer Science");
  });
});

describe("takenEntries and groupByTerm", () => {
  const list = [
    entry({ courseId: "21127", term: "F25" }),
    entry({ courseId: "15122", term: "S26" }),
    entry({ courseId: "15112", source: "exam" }),
    entry({ courseId: "76101" }),
    entry({ courseId: "15122", term: "F25" }),
  ];
  it("keeps CMU courses, newest semester first, unset last", () => {
    expect(takenEntries(list).map((e) => `${e.courseId}@${e.term ?? ""}`)).toEqual([
      "15122@S26",
      "15122@F25",
      "21127@F25",
      "76101@",
    ]);
  });
  it("groups by semester in order", () => {
    expect(groupByTerm(takenEntries(list)).map((g) => [g.term, g.entries.length])).toEqual([
      ["S26", 1],
      ["F25", 2],
      ["", 1],
    ]);
  });
});

describe("filters", () => {
  const list = takenEntries([
    entry({ courseId: "15122", term: "F25" }),
    entry({ courseId: "15619", term: "S26" }),
    entry({ courseId: "21127", term: "F25" }),
    entry({ courseId: "98242" }),
  ]);
  const ids = (f: Partial<typeof ALL_FILTERS>): (string | null)[] =>
    list.filter((e) => matchesFilters(e, { ...ALL_FILTERS, ...f }, factsOf)).map((e) => e.courseId);

  it("match everything by default", () => {
    expect(ids({})).toHaveLength(4);
  });
  it("narrow by department, level and semester", () => {
    expect(ids({ department: "CS" })).toEqual(["15619", "15122"]);
    expect(ids({ level: "G" })).toEqual(["15619"]);
    expect(ids({ semester: "F25" })).toEqual(["15122", "21127"]);
    expect(ids({ semester: NO_TERM })).toEqual(["98242"]);
    expect(ids({ department: "98" })).toEqual(["98242"]);
    expect(ids({ department: "CS", semester: "F25" })).toEqual(["15122"]);
  });
  it("offer only the choices present", () => {
    expect(departmentOptions(list, factsOf).map((o) => o.value)).toEqual(["CS", "98", "MSC"]);
    expect(levelOptions(list, factsOf).map((o) => o.label)).toEqual(["Undergraduate", "Graduate"]);
    expect(semesterOptions(list, label)).toEqual([
      { value: "S26", label: "term S26" },
      { value: "F25", label: "term F25" },
      { value: NO_TERM, label: "Semester not set" },
    ]);
  });
});

describe("currentlyTaking", () => {
  it("lists this semester's courses that can still earn credit", () => {
    const list = [
      entry({ courseId: "15122", term: "F26" }),
      entry({ courseId: "21127", term: "F26", grade: "W" }),
      entry({ courseId: "76101", term: "F26", grade: "A" }),
      entry({ courseId: "15150", term: "S26" }),
      entry({ courseId: "15210", term: "F26", source: "transfer" }),
    ];
    expect(currentlyTaking(list, "F26").map((e) => e.courseId)).toEqual(["15122", "76101"]);
  });
  it("finds scheduled courses missing from the record", () => {
    const list = [
      entry({ courseId: "15122", term: "F26" }),
      entry({ courseId: "21127", term: "S26" }),
    ];
    expect(missingFromRecord(list, "F26", ["15122", "21127", "21127", "76101"])).toEqual([
      "21127",
      "76101",
    ]);
  });
});
