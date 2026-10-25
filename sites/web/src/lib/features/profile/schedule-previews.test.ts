/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../../testing.ts";
import { indexTerm } from "../../core/data.ts";
import type { Schedule, ScheduleEntry } from "../../core/state/store.ts";
import type { Course, Day, Section } from "../../core/types.ts";
import { bySemester, classCount, previewBlocks } from "./schedule-previews.ts";

const sec = (
  name: string,
  days: Day[],
  start: number | null,
  parent: string | null = null,
): Section => ({
  name,
  kind: parent === null ? "lecture" : "section",
  parent,
  mini: null,
  mode: "IPE",
  instructors: [],
  meetings: [
    { days, start, end: start === null ? null : start + 50, building: "GHC", room: "4401" },
  ],
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
const term = indexTerm({
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
    course("15122", [sec("1", [0, 2], 570), sec("A", [4], 540, "1")]),
    course("21127", [sec("1", [1, 3], 660)]),
    course("98012", [{ ...sec("A", [], null), kind: "section" }]),
  ],
});
const entry = (
  courseId: string,
  section: string | null,
  color: number,
  hidden = false,
): ScheduleEntry => ({
  courseId,
  section,
  child: null,
  passFail: false,
  color,
  hidden,
});
const schedule = (entries: ScheduleEntry[], termId = "F26", id = "s"): Schedule => ({
  id,
  termId,
  name: "Schedule 1",
  entries,
  createdAt: 0,
  updatedAt: 0,
});

describe("previewBlocks", () => {
  it("draws each meeting of the chosen sections in the course's color", () => {
    const blocks = previewBlocks(schedule([entry("15122", "1", 4), entry("21127", "1", 7)]), term);
    expect(blocks.map((b) => [b.day, b.start, b.color])).toEqual([
      [0, 570, 4],
      [2, 570, 4],
      [4, 540, 4],
      [1, 660, 7],
      [3, 660, 7],
    ]);
  });

  it("leaves out hidden courses and courses without a set time", () => {
    const blocks = previewBlocks(
      schedule([entry("15122", "1", 4, true), entry("98012", "A", 2), entry("21127", "1", 7)]),
      term,
    );
    expect(blocks.map((b) => b.label)).toEqual(["21-127", "21-127"]);
  });
});

describe("classCount", () => {
  it("counts the courses that aren't hidden", () => {
    expect(
      classCount(
        schedule([entry("15122", "1", 0), entry("21127", "1", 1), entry("98012", "A", 2)]),
      ),
    ).toBe("3 classes");
    expect(classCount(schedule([entry("15122", "1", 0), entry("21127", "1", 1, true)]))).toBe(
      "1 class",
    );
    expect(classCount(schedule([entry("15122", "1", 0, true)]))).toBe("No classes yet");
  });
});

describe("bySemester", () => {
  it("groups schedules under their semesters in the order they come", () => {
    const list = [schedule([], "S27", "a"), schedule([], "F26", "b"), schedule([], "S27", "c")];
    expect(bySemester(list).map((g) => [g.termId, g.schedules.map((s) => s.id)])).toEqual([
      ["S27", ["a", "c"]],
      ["F26", ["b"]],
    ]);
  });
});
