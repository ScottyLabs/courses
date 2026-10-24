/* oxlint-disable typescript/no-floating-promises, eslint/max-lines-per-function -- node:test awaits describe and it itself; a describe groups many cases */
import { describe, it } from "node:test";
import { expect } from "../../testing.ts";
import { indexTerm } from "../../core/data.ts";
import { buildScheduleView } from "../../core/state/schedule-view.ts";
import type { Schedule, ScheduleEntry } from "../../core/state/store.ts";
import type { Course, Day, Section, TermData } from "../../core/types.ts";
import {
  conflictCount,
  courseListText,
  fileName,
  overlayBlocks,
  plannerBlocks,
  tbaItems,
  type CalendarSelection,
} from "./planner.ts";

const sec = (
  name: string,
  days: Day[],
  start: number | null,
  parent: string | null = null,
  mini: number | null = null,
): Section => ({
  name,
  kind: parent === null ? "lecture" : "section",
  parent,
  mini,
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
const data: TermData = {
  term: {
    id: "F26",
    name: "Fall 2026",
    season: "fall",
    year: 2026,
    minis: [
      { number: 1, name: "Mini 1", start: "2026-08-24", end: "2026-10-09" },
      { number: 2, name: "Mini 2", start: "2026-10-19", end: "2026-12-04" },
    ],
    classesBegin: "2026-08-24",
    classesEnd: "2026-12-04",
    noClasses: [],
  },
  courses: [
    course("15122", [sec("1", [1, 3], 570), sec("A", [4], 540, "1"), sec("2", [1, 3], 660)]),
    course("21127", [sec("1", [1, 3], 570)]),
    course("98012", [{ ...sec("A", [], null), kind: "section" }]),
    course("70100", [sec("A", [0], 600, null, 2)]),
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
const schedule = (entries: ScheduleEntry[]): Schedule => ({
  id: "s",
  termId: "F26",
  name: "Mine",
  entries,
  createdAt: 0,
  updatedAt: 0,
});
const noop = { select: () => {}, switchSection: () => {}, switchChild: () => {} };

describe("plannerBlocks", () => {
  const view = buildScheduleView(
    schedule([
      entry("15122", "1", "A"),
      entry("21127", "1"),
      entry("98012", "A"),
      entry("70100", "A"),
    ]),
    term,
  );

  it("places sections, marks conflicts and counts them", () => {
    const blocks = plannerBlocks(view, term, {
      selected: null,
      preview: null,
      mini: null,
      on: noop,
    });
    expect(blocks.filter((b) => b.label === "15-122").map((b) => [b.section, b.day])).toEqual([
      ["Lec 1", 1],
      ["Lec 1", 3],
      ["Rec A", 4],
    ]);
    expect(blocks.filter((b) => b.conflict === true).length).toBe(5);
    expect(conflictCount(view)).toBe(1);
  });

  it("outlines a selected course's other lectures and switches to one", () => {
    const switched: string[] = [];
    const selected: CalendarSelection = { courseId: "15122", level: "section" };
    const blocks = plannerBlocks(view, term, {
      selected,
      preview: null,
      mini: null,
      on: {
        ...noop,
        switchSection: (id, name) => {
          switched.push(`${id}:${name}`);
        },
      },
    });
    const ghosts = blocks.filter((b) => b.variant === "ghost");
    expect(ghosts.map((b) => b.section)).toEqual(["Lec 2", "Lec 2"]);
    ghosts[0]?.onClick?.();
    expect(switched).toEqual(["15122:2"]);
  });

  it("previews a course's lectures and leaves out other minis", () => {
    const blocks = plannerBlocks(buildScheduleView(schedule([entry("70100", "A")]), term), term, {
      selected: null,
      preview: { courseId: "15122", sections: null },
      mini: 1,
      on: noop,
    });
    expect(blocks.map((b) => `${b.variant ?? "solid"} ${b.section}`)).toEqual([
      "preview Lec 1",
      "preview Lec 1",
      "preview Lec 2",
      "preview Lec 2",
    ]);
  });

  it("shows a friend's schedule labeled with their name", () => {
    const blocks = overlayBlocks({ entries: [entry("21127", "1")] }, term, "Alex", null);
    expect(blocks.map((b) => [b.label, b.section, b.variant])).toEqual([
      ["21-127", "Alex", "preview"],
      ["21-127", "Alex", "preview"],
    ]);
  });
});

describe("schedule lists", () => {
  const view = buildScheduleView(schedule([entry("15122", "1"), entry("98012", "A")]), term);

  it("lists courses without a set time", () => {
    expect(tbaItems(view, "F26").map((i) => [i.label, i.detail, i.href])).toEqual([
      ["98-012", "Sec A \u00B7 12 units", "/course/98-012?term=F26"],
    ]);
  });

  it("writes the schedule as text", () => {
    expect(courseListText(view, "Mine", "Fall 2026").split("\n")).toEqual([
      "Mine (Fall 2026)",
      "15-122 Course 15122, 12 units, Lec 1: TR 9:30\u201310:20 AM; Rec A: F 9:00\u20139:50 AM",
      "98-012 Course 98012, 12 units, Sec A: Time TBA",
      "Total: 24 units",
    ]);
  });

  it("makes file names safe", () => {
    expect(fileName('a/b: "c"')).toBe("a b c");
    expect(fileName("  ")).toBe("schedule");
  });
});
