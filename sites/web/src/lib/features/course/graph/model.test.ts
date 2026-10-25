/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../../../testing.ts";
import { summarizeCheck } from "../../../core/record/check-summary.ts";
import { checkCourse, createContext, type RequisiteContext } from "../../../core/requisites.ts";
import { buildGraph } from "./model.ts";
import { courseStatus, graphSummary } from "./status.ts";
import {
  any,
  catalog,
  course,
  courses,
  done,
  entry,
  find,
  group,
  INDEX,
  options,
  planned,
} from "./test-fixtures.ts";

describe("buildGraph: structure", () => {
  it("mirrors the requisite expression, flattening groups of one", () => {
    const top = group(buildGraph(options()).requires);
    expect(top.label).toBe("AND");
    expect(top.items.map((i) => i.kind)).toEqual(["course", "course", "group"]);
    expect(group(top.items[2]).label).toBe("OR");
    expect(find(buildGraph(options()), "15210").minGrade).toBe("C");
    // 15-122's "one of 15-112" is just 15-112, beside its corequisite.
    const fork = group(buildGraph(options({ rootId: "15122", unlocks: [] })).requires);
    expect(fork.op).toBe("fork");
    expect(fork.items[0]?.kind).toBe("course");
    expect(group(fork.items[1])).toMatchObject({ label: "WITH", relation: "coreq" });
  });

  it("expands a course's own requisites only when asked, and stops at cycles", () => {
    const closed = buildGraph(options());
    expect(find(closed, "15210")).toMatchObject({ expandable: true, requires: null });
    expect(find(closed, "21241").expandable).toBe(false);
    const open = buildGraph(options({ autoDepth: 3 }));
    expect(find(open, "15150").depth).toBe(2);
    expect(find(open, "15122").depth).toBe(3);
    expect(courses(open).some((n) => n.id === "15112")).toBe(true);
    const looped = catalog(entry("10001", course("10002")), entry("10002", course("10001")));
    const cycle = buildGraph(
      options({ index: looped, rootId: "10001", unlocks: [], autoDepth: 5 }),
    );
    expect(courses(cycle).find((n) => n.id === "10001")).toMatchObject({
      cycle: true,
      expandable: false,
    });
  });
});

describe("buildGraph: expanding", () => {
  it("expands each course once under Expand all, and any one the student opens", () => {
    const twice = courses(buildGraph(options({ autoDepth: 9 }))).filter((n) => n.id === "15122");
    expect(twice.map((n) => [n.expanded, n.shownElsewhere === true])).toEqual([
      [true, false],
      [false, true],
    ]);
    const both = buildGraph(options({ autoDepth: 9, expanded: new Set([twice[1]?.key ?? ""]) }));
    expect(courses(both).filter((n) => n.id === "15122" && n.expanded)).toHaveLength(2);
    const collapsed = buildGraph(
      options({ autoDepth: 9, collapsed: new Set([twice[0]?.key ?? ""]) }),
    );
    expect(courses(collapsed).filter((n) => n.id === "15122" && n.expanded)).toHaveLength(1);
  });

  it("caps long groups behind a more button, and reveals them", () => {
    const many = Array.from({ length: 12 }, (_, i) => course(`210${String(i).padStart(2, "0")}`));
    const index = catalog(entry("99999", any(...many)));
    const capped = group(
      buildGraph(options({ index, rootId: "99999", unlocks: [], limits: { group: 8 } })).requires,
    );
    expect(capped.items).toHaveLength(9);
    expect(capped.items.at(-1)).toMatchObject({ kind: "more", hidden: 4, label: "+4 more" });
    const shown = buildGraph(
      options({ index, rootId: "99999", unlocks: [], revealed: new Set([capped.key]) }),
    );
    expect(group(shown.requires).items.at(-1)).toMatchObject({
      kind: "more",
      less: true,
      label: "Show fewer",
    });
  });
});

describe("buildGraph: the student's standing", () => {
  it("colors courses and groups from the same evaluation the course pages use", () => {
    const ctx = createContext({
      credits: [done("15210", "B"), done("21241", "D"), planned("21228")],
    });
    const model = buildGraph(options({ ctx }));
    expect(find(model, "15210")).toMatchObject({ status: "met", note: "Taken in Fall 2025, B" });
    expect(find(model, "21241").status).toBe("grade-low");
    expect(find(model, "21228")).toMatchObject({
      status: "planned",
      note: "Planned for Spring 2027",
    });
    expect(find(model, "15251").status).toBe("missing");
    const top = group(model.requires);
    expect(top.state).toBe("missing");
    expect(group(top.items[2]).state).toBe("planned");
    // The same verdict as the course page's requisite line.
    expect(
      summarizeCheck(checkCourse("15451", INDEX.courses["15451"], ctx), INDEX.courses["15451"])
        .level,
    ).toBe("problem");
    expect(graphSummary("15451", INDEX.courses["15451"], ctx).short).toBe("1 missing");
  });

  it("leaves missing courses plain while nothing is on record", () => {
    const model = buildGraph(options({ ctx: createContext({ credits: [] }), unchecked: true }));
    expect(find(model, "15210").status).toBeUndefined();
    expect(group(model.requires).state).toBeUndefined();
  });
});

describe("buildGraph: waivers and plans", () => {
  it("marks waived requisites and courses in progress", () => {
    const waiver = { courseId: "21127", timing: "completed" as const, source: "waiver" as const };
    const ctx: RequisiteContext = {
      ...createContext({ credits: [done("15122", undefined, "F26"), waiver] }),
      overrides: { "15150": { prereqs: true } },
    };
    const model = buildGraph(options({ rootId: "15150", unlocks: [], ctx }));
    expect(find(model, "15122")).toMatchObject({
      status: "in-progress",
      note: "Taking it in Fall 2026",
    });
    expect(find(model, "21127").status).toBe("waived");
    expect(model.requires?.link).toBe("waived");
    expect(graphSummary("15150", INDEX.courses["15150"], ctx).short).toBe("Requisites waived");
  });

  it("summarizes what planned courses would meet", () => {
    const ctx = createContext({ credits: [done("15210"), done("21241"), planned("15251")] });
    expect(graphSummary("15451", INDEX.courses["15451"], ctx).short).toBe(
      "Met once 15-251 is done",
    );
  });
});

describe("buildGraph: unlocks", () => {
  it("marks the courses the student could take next", () => {
    const model = buildGraph(options({ ctx: createContext({ credits: [done("21241")] }) }));
    expect(find(model, "15456").ready).toBe(true);
    expect(find(model, "15457").ready).toBe(true);
    const without = buildGraph(options({ ctx: createContext({ credits: [done("15122")] }) }));
    expect(find(without, "15456").ready).toBe(true);
    expect(find(without, "15457").ready).toBeUndefined();
  });

  it("groups long lists by department, the biggest first", () => {
    const unlocks = [
      ...Array.from({ length: 9 }, (_, i) => ({
        id: `1530${i}`,
        via: "prereq" as const,
        required: true,
      })),
      ...Array.from({ length: 5 }, (_, i) => ({
        id: `2130${i}`,
        via: "prereq" as const,
        required: false,
      })),
    ];
    const depts =
      buildGraph(options({ unlocks, limits: { flat: 12, department: 4 } })).unlocks?.items ?? [];
    expect(depts.map((d) => [group(d).label, group(d).count])).toEqual([
      ["Dept 15", 9],
      ["Dept 21", 5],
    ]);
    expect(group(depts[0]).items.at(-1)).toMatchObject({ kind: "more", label: "+5 more" });
  });
});

describe("courseStatus", () => {
  const met = {
    status: "met" as const,
    node: course("15122"),
    match: { credit: done("15122", "A"), grade: "ok" as const },
  };

  it("says what a status rests on", () => {
    expect(courseStatus(met, "F26")).toEqual({ status: "met", note: "Taken in Fall 2025, A" });
    const credit = { courseId: "15122", timing: "completed" as const, source: "transfer" as const };
    expect(courseStatus({ ...met, match: { credit, grade: "ok" } }, "F26").note).toBe(
      "Transfer credit",
    );
    const low = {
      status: "missing" as const,
      node: course("15122", "C"),
      gradeTooLow: { grade: "D", minGrade: "C" },
    };
    expect(courseStatus(low, "F26")).toEqual({
      status: "grade-low",
      note: "You have D; it needs C or better",
    });
  });

  it("names an equivalent course that stands in", () => {
    expect(courseStatus({ ...met, match: { ...met.match, via: "15151" } }, "F26").note).toBe(
      "Taken in Fall 2025, A (as 15-151)",
    );
  });
});
