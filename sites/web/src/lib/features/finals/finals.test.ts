/* oxlint-disable typescript/no-floating-promises, eslint/max-lines-per-function -- node:test awaits describe and it itself; a describe groups many cases */
import { describe, it } from "node:test";
import { expect } from "../../testing.ts";
import type { RegistrarFinal } from "../../data/documents.ts";
import { indexTerm } from "../../core/data.ts";
import { resolveEntries } from "../../core/state/schedule-view.ts";
import type { ScheduleEntry } from "../../core/state/store.ts";
import type { Course, Day, Section, TermData } from "../../core/types.ts";
import { examDay, finalWarnings, finalsIcs, fromSio, matchFinals, type MyFinal } from "./finals.ts";

const sec = (name: string, parent: string | null = null, days: Day[] = [1, 3]): Section => ({
  name,
  kind: parent === null ? "lecture" : "section",
  parent,
  mini: null,
  mode: "IPE",
  instructors: [],
  meetings: [{ days, start: 600, end: 650, building: null, room: null }],
});
const course = (id: string, sections: Section[]): Course => ({
  id,
  code: `${id.slice(0, 2)}-${id.slice(2)}`,
  title: `Course ${id}`,
  units: 12,
  dept: "CS",
  level: "U",
  instructors: [],
  sections,
  crossListed: [],
});
const data: TermData = {
  term: {
    id: "F26",
    name: "Fall 2026",
    season: "fall",
    year: 2026,
    minis: [],
    classesBegin: "2026-08-24",
    classesEnd: "2026-12-04",
    noClasses: [],
  },
  courses: [
    course("15122", [sec("1"), sec("A", "1"), sec("2"), sec("B", "2")]),
    course("21127", [sec("1"), sec("2")]),
    course("76101", [sec("A"), sec("B")]),
    course("98012", [sec("A")]),
  ],
};
const term = indexTerm(data);
const entry = (
  courseId: string,
  section: string | null,
  child: string | null = null,
): ScheduleEntry => ({
  courseId,
  section,
  child,
  passFail: false,
  color: 0,
  hidden: false,
});
const exam = (courseId: string, section: string, date: string, start: number): RegistrarFinal => ({
  courseId,
  section,
  title: "EXAM",
  date,
  start,
  end: start + 180,
  mode: "In Person",
  location: "To Be Announced",
  instructors: [],
});
const resolved = (entries: ScheduleEntry[]) =>
  resolveEntries({ id: "s", termId: "F26", name: "S", entries, createdAt: 0, updatedAt: 0 }, term);

describe("matchFinals", () => {
  it("matches the chosen lecture's exam, or the course's when every section shares one", () => {
    const finals = [
      exam("15122", "1", "2026-12-07", 510),
      exam("15122", "2", "2026-12-08", 510),
      exam("21127", "1", "2026-12-09", 780),
      exam("21127", "2", "2026-12-09", 780),
    ];
    const mine = matchFinals(
      resolved([entry("15122", "2", "B"), entry("21127", null), entry("76101", "A")]),
      finals,
    );
    expect(mine.map((f) => [f.code, f.match, f.exam?.date ?? null])).toEqual([
      ["15-122", "section", "2026-12-08"],
      ["21-127", "course", "2026-12-09"],
      ["76-101", "none", null],
    ]);
  });

  it("asks for a section when exams differ by section, and uses the SIO import when the registrar has none", () => {
    const finals = [exam("15122", "1", "2026-12-07", 510), exam("15122", "2", "2026-12-08", 510)];
    const sio = [
      {
        courseId: "98012",
        courseCode: "98-012",
        title: "StuCo final",
        startsAt: "2026-12-10T18:30:00Z",
        endsAt: "2026-12-10T20:30:00Z",
        location: "DH 2210",
      },
    ];
    const mine = matchFinals(resolved([entry("15122", null), entry("98012", "A")]), finals, sio);
    expect(mine.map((f) => f.match)).toEqual(["pick", "sio"]);
    // 18:30 UTC in December is 1:30 PM in Pittsburgh.
    expect(mine[1]?.exam?.start).toBe(810);
  });
});

const mine = (code: string, date: string, start: number): MyFinal => ({
  courseId: code.replace("-", ""),
  code,
  title: code,
  color: 0,
  match: "section",
  exam: {
    courseId: code.replace("-", ""),
    code,
    section: "1",
    title: code,
    date,
    start,
    end: start + 180,
    location: null,
    mode: "In Person",
    source: "registrar",
  },
});

describe("finalWarnings", () => {
  it("flags overlaps, back-to-back exams and three in 24 hours", () => {
    const warnings = finalWarnings([
      mine("15-122", "2026-12-07", 510),
      mine("21-127", "2026-12-07", 780),
      mine("76-101", "2026-12-07", 1050),
      mine("15-150", "2026-12-09", 510),
      mine("15-151", "2026-12-09", 600),
    ]);
    expect(warnings.map((w) => [w.kind, w.codes])).toEqual([
      ["back-to-back", ["15-122", "21-127"]],
      ["back-to-back", ["21-127", "76-101"]],
      ["overlap", ["15-150", "15-151"]],
      ["three-in-24", ["15-122", "21-127", "76-101"]],
    ]);
  });

  it("counts 24 hours across days and reports each run once", () => {
    const warnings = finalWarnings([
      mine("15-122", "2026-12-07", 1050),
      mine("21-127", "2026-12-08", 510),
      mine("76-101", "2026-12-08", 780),
      mine("15-150", "2026-12-08", 1050),
    ]);
    expect(warnings.filter((w) => w.kind === "three-in-24").map((w) => w.codes)).toEqual([
      ["15-122", "21-127", "76-101"],
      ["21-127", "76-101", "15-150"],
    ]);
  });
});

describe("finalsIcs", () => {
  it("writes one event per exam in Pittsburgh time", () => {
    const one = mine("15-122", "2026-12-07", 510).exam;
    if (one === null) throw new Error("no exam");
    const ics = finalsIcs([one], "Finals", new Date("2026-10-01T00:00:00Z"));
    expect(ics).toContain("DTSTART;TZID=America/New_York:20261207T083000");
    expect(ics).toContain("DTEND;TZID=America/New_York:20261207T113000");
    expect(ics).toContain("SUMMARY:15-122 final exam");
  });

  it("names the day", () => {
    expect(examDay("2026-12-07")).toBe("Monday, December 7");
  });

  it("reads SIO times in UTC", () => {
    const e = fromSio({
      courseId: "15122",
      courseCode: "15-122",
      title: "x",
      startsAt: "2026-05-04T12:30:00Z",
      endsAt: "2026-05-04T15:30:00Z",
      location: "",
    });
    expect([e.date, e.start, e.end, e.location]).toEqual(["2026-05-04", 510, 690, null]);
  });
});
