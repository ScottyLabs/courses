/* oxlint-disable typescript/no-floating-promises, typescript/no-non-null-assertion, typescript/no-unnecessary-type-assertion, eslint/max-lines-per-function -- Courseweave's state/plan.test.ts, ported to these stores (decision 14); node:test awaits describe and it itself */
import { beforeEach, describe, it } from "node:test";
import { effectiveTermCourses, linkedCourseId } from "../core/plan.ts";
import { migratePlans } from "../core/state/plan.ts";
import type { Schedule } from "../core/state/store.ts";
import { expect } from "../testing.ts";
import type { Personal } from "./personal.ts";
import { draftStores } from "./test-env.ts";

const entry = (courseId: string, extra: Partial<Schedule["entries"][number]> = {}) => ({
  courseId,
  section: "1",
  child: null,
  color: 0,
  hidden: false,
  passFail: false,
  ...extra,
});

let stores: Personal;

function setup() {
  stores = draftStores();
  const schedule: Schedule = {
    id: "sch1",
    termId: "S27",
    name: "Schedule 1",
    entries: [entry("15213"), entry("99101")],
    createdAt: 1,
    updatedAt: 1,
  };
  stores.schedules.store.create(schedule);
  stores.schedules.setActiveSchedule("S27", "sch1");
  const planId = stores.plans.createPlan({ startTerm: "F26", terms: ["F26", "S27"] });
  stores.plans.addCourse(planId, "S27", "76101");
  stores.plans.addCourse(planId, "S27", "15213", { units: 9 });
  return planId;
}

const plan = (id: string) => stores.plans.items[id]!;
const sch1 = () => stores.schedules.items.sch1!;
const scheduleIds = () => sch1().entries.map((e) => e.courseId);
const effective = (id: string, term: string) =>
  effectiveTermCourses(plan(id), stores.schedules.items)
    .get(term)!
    .courses.map((c) => c.courseId);
const links = () => stores.links;

describe("syncing a plan semester with a schedule", () => {
  beforeEach(() => {
    stores = draftStores();
  });

  it("merges planned courses into the schedule and snapshots the result", () => {
    const id = setup();
    links().linkTermToSchedule(plan(id), "S27", sch1(), "merge");
    const term = plan(id).terms.find((t) => t.term === "S27")!;
    expect(term.link).toEqual({ scheduleId: "sch1" });
    expect(scheduleIds()).toEqual(["15213", "99101", "76101"]);
    expect(term.courses.map((c) => c.courseId)).toEqual(["15213", "99101", "76101"]);
    // The unit override for 15-213 survives into the snapshot and the effective courses.
    expect(term.courses.find((c) => c.courseId === "15213")?.units).toBe(9);
    expect(
      effectiveTermCourses(plan(id), stores.schedules.items).get("S27")!.courses[0]!.units,
    ).toBe(9);
  });

  it("uses the schedule as is when asked, dropping the other planned courses", () => {
    const id = setup();
    links().linkTermToSchedule(plan(id), "S27", sch1(), "as-is");
    expect(scheduleIds()).toEqual(["15213", "99101"]);
    expect(effective(id, "S27")).toEqual(["15213", "99101"]);
  });

  it("routes adds, removes, moves and pass/fail to the synced schedule", () => {
    const id = setup();
    links().linkTermToSchedule(plan(id), "S27", sch1(), "as-is");

    expect(links().addToTerm(plan(id), "S27", "15251")).toBe(linkedCourseId("sch1", "15251"));
    expect(scheduleIds()).toContain("15251");

    links().removeFromPlan(plan(id), linkedCourseId("sch1", "99101"));
    expect(scheduleIds()).not.toContain("99101");
    expect(
      plan(id)
        .terms.find((t) => t.term === "S27")!
        .courses.map((c) => c.courseId),
    ).not.toContain("99101");

    // Out of the synced semester into a plain one, and back.
    links().moveInPlan(plan(id), linkedCourseId("sch1", "15251"), "F26");
    expect(scheduleIds()).not.toContain("15251");
    expect(effective(id, "F26")).toEqual(["15251"]);
    const f26Id = plan(id).terms.find((t) => t.term === "F26")!.courses[0]!.id;
    links().moveInPlan(plan(id), f26Id, "S27");
    expect(effective(id, "F26")).toEqual([]);
    expect(scheduleIds()).toContain("15251");

    links().setPlanPassFail(plan(id), linkedCourseId("sch1", "15213"), true);
    expect(sch1().entries.find((e) => e.courseId === "15213")?.passFail).toBe(true);

    links().setPlanUnits(plan(id), linkedCourseId("sch1", "15251"), 6);
    expect(
      effectiveTermCourses(plan(id), stores.schedules.items)
        .get("S27")!
        .courses.find((c) => c.courseId === "15251")?.units,
    ).toBe(6);
  });

  it("keeps the schedule courses as planned courses when syncing stops", () => {
    const id = setup();
    links().linkTermToSchedule(plan(id), "S27", sch1(), "as-is");
    stores.schedules.addCourse("sch1", "21127");
    links().unlinkTerm(plan(id), "S27");
    const term = plan(id).terms.find((t) => t.term === "S27")!;
    expect(term.link).toBeNull();
    expect(term.courses.map((c) => c.courseId)).toEqual(["15213", "99101", "21127"]);
  });

  it("falls back to the snapshot when the schedule is deleted", () => {
    const id = setup();
    links().linkTermToSchedule(plan(id), "S27", sch1(), "merge");
    stores.schedules.deleteSchedule("sch1");
    const eff = effectiveTermCourses(plan(id), stores.schedules.items).get("S27")!;
    expect(eff.missing).toBe(true);
    expect(eff.courses.map((c) => c.courseId)).toEqual(["15213", "99101", "76101"]);
    // Edits then go to the plan itself.
    links().addToTerm(plan(id), "S27", "15251");
    expect(effective(id, "S27")).toContain("15251");
  });
});

describe("plan store migration", () => {
  it("gives v1 semesters an empty link", () => {
    const v1 = {
      plans: {
        p: {
          id: "p",
          name: "Plan",
          programs: [],
          startTerm: "F25",
          terms: [{ term: "F25", courses: [] }],
          createdAt: 0,
          updatedAt: 0,
        },
      },
      activePlanId: "p",
    };
    const migrated = migratePlans(v1, 1);
    expect(migrated.plans.p!.terms[0]).toEqual({ term: "F25", courses: [], link: null });
    expect(migrated.activePlanId).toBe("p");
  });
});

describe("reordering courses in a semester", () => {
  const ids = (id: string, term: string) =>
    plan(id)
      .terms.find((t) => t.term === term)!
      .courses.map((c) => c.courseId);
  const courseId = (id: string, term: string, code: string) =>
    plan(id)
      .terms.find((t) => t.term === term)!
      .courses.find((c) => c.courseId === code)!.id;

  it("moves a course before another or to the end of its own semester", () => {
    const id = setup();
    stores.plans.addCourse(id, "S27", "21241");
    expect(ids(id, "S27")).toEqual(["76101", "15213", "21241"]);
    links().moveInPlan(plan(id), courseId(id, "S27", "21241"), "S27", courseId(id, "S27", "76101"));
    expect(ids(id, "S27")).toEqual(["21241", "76101", "15213"]);
    links().moveInPlan(plan(id), courseId(id, "S27", "21241"), "S27", null);
    expect(ids(id, "S27")).toEqual(["76101", "15213", "21241"]);
    // Without a position, "moving" to its own semester does nothing.
    links().moveInPlan(plan(id), courseId(id, "S27", "21241"), "S27");
    expect(ids(id, "S27")).toEqual(["76101", "15213", "21241"]);
  });

  it("places a course moved from another semester where it was dropped", () => {
    const id = setup();
    stores.plans.addCourse(id, "F26", "15122");
    links().moveInPlan(plan(id), courseId(id, "F26", "15122"), "S27", courseId(id, "S27", "15213"));
    expect(ids(id, "S27")).toEqual(["76101", "15122", "15213"]);
    expect(ids(id, "F26")).toEqual([]);
  });

  it("reorders a synced semester's schedule, leaving hidden entries in order", () => {
    const id = setup();
    stores.schedules.store.edit("sch1", (s) => ({
      ...s,
      entries: [entry("15213"), entry("99999", { hidden: true }), entry("99101")],
    }));
    links().linkTermToSchedule(plan(id), "S27", sch1(), "as-is");
    links().placeInTerm(
      plan(id),
      linkedCourseId("sch1", "99101"),
      "S27",
      linkedCourseId("sch1", "15213"),
    );
    expect(scheduleIds()).toEqual(["99101", "15213", "99999"]);
    expect(effective(id, "S27")).toEqual(["99101", "15213"]);
  });
});

describe("createPlan", () => {
  it("takes semesters with their courses, as an import gives them", () => {
    stores = draftStores();
    const id = stores.plans.createPlan({
      name: "Stellic plan",
      startTerm: "F26",
      terms: [
        { term: "S27", courses: [{ courseId: "15213" }, { courseId: "10999", units: 12 }] },
        "F26",
      ],
    });
    const p = plan(id);
    expect(stores.plans.activeId).toBe(id);
    expect(p.name).toBe("Stellic plan");
    expect(p.terms.map((t) => [t.term, t.courses.map((c) => [c.courseId, c.units])])).toEqual([
      ["F26", []],
      [
        "S27",
        [
          ["15213", undefined],
          ["10999", 12],
        ],
      ],
    ]);
    expect(new Set(p.terms[1]!.courses.map((c) => c.id)).size).toBe(2);
  });
});
