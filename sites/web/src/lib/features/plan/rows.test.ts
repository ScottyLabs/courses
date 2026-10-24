/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import type { CatalogIndex } from "../../core/catalog-types.ts";
import type { PlanCheck } from "../../core/plan.ts";
import { expect } from "../../testing.ts";
import { courseRowInfo, dropsInPlace, nudgeTarget } from "./rows.ts";

// oxlint-disable-next-line typescript/no-unsafe-type-assertion -- a partial catalog is enough here
const catalog = {
  courses: {
    "15122": {
      id: "15122",
      code: "15-122",
      title: "Imperative",
      units: 12,
      offered: ["fall", "spring"],
    },
    "99101": {
      id: "99101",
      code: "99-101",
      title: "Computing @ Carnegie Mellon",
      units: 3,
      passFail: true,
    },
    "15400": { id: "15400", code: "15-400", title: "Research", units: null, offered: ["fall"] },
  },
} as unknown as CatalogIndex;

const unitsOf = (id: string) => catalog.courses[id]?.units ?? null;

function check(level: PlanCheck["check"]["level"], messages: string[] = []): PlanCheck {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- only the fields the row reads
  return {
    planCourseId: "p1",
    courseId: "15122",
    term: "S27",
    check: {
      level,
      issues: messages.map((message) => ({
        kind: "prereq",
        severity: "error",
        message,
        courses: [],
      })),
    },
  } as unknown as PlanCheck;
}

describe("courseRowInfo", () => {
  const base = { term: "S27", current: "F26", catalog, unitsOf, scheduled: undefined };

  it("reads units, pass/fail and requisite problems for a future semester", () => {
    const info = courseRowInfo({
      ...base,
      course: { id: "p1", courseId: "15122" },
      check: check("problem", ["Missing prerequisite: 15-112"]),
    });
    expect(info.units).toBe(12);
    expect(info.passFail).toBe(false);
    expect(info.level).toBe("problem");
    expect(info.statusText).toBe("Missing prerequisite: 15-112");
    expect(info.offeringWarning).toBe(null);
  });

  it("marks pass/fail-only courses and elected pass/fail", () => {
    const only = courseRowInfo({
      ...base,
      course: { id: "p2", courseId: "99101" },
      check: undefined,
    });
    expect(only.passFailOnly).toBe(true);
    expect(only.passFail).toBe(true);
    const elected = courseRowInfo({
      ...base,
      course: { id: "p3", courseId: "15122", passFail: true },
      check: undefined,
    });
    expect(elected.passFail).toBe(true);
    expect(elected.passFailOnly).toBe(false);
  });
});

describe("courseRowInfo offering", () => {
  const base = { term: "S27", current: "F26", catalog, unitsOf, scheduled: undefined };

  it("warns about courses not usually offered that season, or missing from the schedule", () => {
    const research = courseRowInfo({
      ...base,
      course: { id: "p4", courseId: "15400", units: 9 },
      check: undefined,
    });
    expect(research.units).toBe(9);
    expect(research.variable).toBe(true);
    expect(research.offeringWarning).toBe("Not usually offered this season");
    const scheduled = courseRowInfo({
      ...base,
      scheduled: new Set(["15400"]),
      course: { id: "p5", courseId: "15122" },
      check: check("ok"),
    });
    expect(scheduled.offeringWarning).toBe("Not on this semester\u2019s schedule");
    expect(scheduled.statusText).toBe("Requisites met");
  });

  it("checks nothing for past semesters", () => {
    const past = courseRowInfo({
      ...base,
      term: "F25",
      course: { id: "p6", courseId: "15400" },
      check: check("problem"),
    });
    expect(past.level).toBe(null);
    expect(past.offeringWarning).toBe(null);
  });
});

describe("dropsInPlace", () => {
  const planned = [{ id: "a" }, { id: "b" }, { id: "c" }];

  it("is true on the course's own slot", () => {
    expect(dropsInPlace(planned, "b", "b")).toBe(true);
    expect(dropsInPlace(planned, "b", "c")).toBe(true);
    expect(dropsInPlace(planned, "c", null)).toBe(true);
  });

  it("is false anywhere else, or for a course from another semester", () => {
    expect(dropsInPlace(planned, "b", "a")).toBe(false);
    expect(dropsInPlace(planned, "a", null)).toBe(false);
    expect(dropsInPlace(planned, "x", null)).toBe(false);
  });
});

describe("nudgeTarget", () => {
  const planned = [{ id: "a" }, { id: "b" }, { id: "c" }];

  it("moves up before the previous course and down past the next one", () => {
    expect(nudgeTarget(planned, 1, "up")).toBe("a");
    expect(nudgeTarget(planned, 0, "down")).toBe("c");
    expect(nudgeTarget(planned, 1, "down")).toBe(null);
  });

  it("stops at the edges", () => {
    expect(nudgeTarget(planned, 0, "up")).toBe(undefined);
    expect(nudgeTarget(planned, 2, "down")).toBe(undefined);
  });
});
