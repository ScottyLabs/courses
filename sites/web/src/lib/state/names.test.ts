/* oxlint-disable typescript/no-floating-promises, eslint/max-lines-per-function -- node:test awaits describe and it itself; a describe groups many cases */
import { describe, it } from "node:test";
import { expect } from "../testing.ts";
import { distinctName } from "./names.ts";
import { DEBOUNCE_MS, harness } from "./test-env.ts";

describe("distinctName", () => {
  it("keeps a name nobody uses", () => {
    expect(distinctName("Schedule 1", ["Schedule 2"], "Schedule")).toBe("Schedule 1");
  });

  it("moves a default name to the next free one", () => {
    expect(distinctName("Schedule 1", ["Schedule 1"], "Schedule")).toBe("Schedule 2");
    expect(distinctName("Plan 2", ["Plan 1", "Plan 2", "Plan 3"], "Plan")).toBe("Plan 4");
  });

  it("numbers any other name", () => {
    expect(distinctName("Fall plan", ["Fall plan"], "Plan")).toBe("Fall plan 2");
    expect(distinctName("Fall plan", ["Fall plan", "Fall plan 2"], "Plan")).toBe("Fall plan 3");
  });
});

describe("drafts saved on signing in", () => {
  it("get names the account's schedules and plans don't have, and empty defaults are dropped", async () => {
    const h = await harness();
    // Signed out: the student fills Spring's "Schedule 1", visits Fall (an empty "Schedule 1") and drafts "Plan 1".
    const spring = h.personal.schedules.ensureSchedule("S27");
    h.personal.schedules.addCourse(spring, "15122");
    const fall = h.personal.schedules.ensureSchedule("F26");
    h.personal.plans.createPlan({ startTerm: "F26" });
    expect(h.personal.schedules.get(spring)?.name).toBe("Schedule 1");
    expect(h.personal.schedules.get(fall)?.name).toBe("Schedule 1");
    expect(h.personal.plans.list.map((p) => p.name)).toEqual(["Plan 1"]);

    // The account already has a "Schedule 1" in both semesters and a "Plan 1".
    const stored = {
      entries: [],
      visibility: "private",
      shareId: null,
      revision: 2,
      createdAt: 1,
      updatedAt: 1,
    };
    h.server.schedules.records.set("s27", {
      ...stored,
      id: "s27",
      termId: "S27",
      name: "Schedule 1",
    });
    h.server.schedules.records.set("f26", {
      ...stored,
      id: "f26",
      termId: "F26",
      name: "Schedule 1",
    });
    h.server.plans.records.set("p1", {
      id: "p1",
      name: "Plan 1",
      programs: [],
      startTerm: "F25",
      terms: [],
      revision: 1,
      createdAt: 1,
      updatedAt: 1,
    });

    h.server.signedIn = true;
    const back = await h.reload();
    expect(back.personal.mode).toBe("account");
    back.personal.acceptDrafts();
    await back.env.advance(DEBOUNCE_MS);

    expect(back.personal.schedules.forTerm("S27").map((s) => s.name)).toEqual([
      "Schedule 1",
      "Schedule 2",
    ]);
    expect(back.server.schedules.records.get(spring)).toMatchObject({
      name: "Schedule 2",
      termId: "S27",
    });
    // The empty Fall default is left out: Fall shows the account's own "Schedule 1".
    expect(back.personal.schedules.forTerm("F26").map((s) => s.name)).toEqual(["Schedule 1"]);
    expect(back.server.schedules.records.has(fall)).toBe(false);
    expect(back.personal.schedules.activeId("F26")).toBe("f26");
    expect(back.personal.plans.list.map((p) => p.name)).toEqual(["Plan 1", "Plan 2"]);
    const planNames = [...back.server.plans.records.values()].map((p) => String(p.name));
    expect(planNames.toSorted((a, b) => a.localeCompare(b))).toEqual(["Plan 1", "Plan 2"]);
  });

  it("keep an empty schedule on the device when the account has none for its semester", async () => {
    const h = await harness();
    const id = h.personal.schedules.ensureSchedule("S27");
    h.server.signedIn = true;
    const back = await h.reload();
    await back.env.advance(10 * DEBOUNCE_MS);
    expect(back.personal.schedules.forTerm("S27").map((s) => s.id)).toEqual([id]);
    expect(back.server.schedules.records.size).toBe(0);
    back.personal.schedules.addCourse(id, "15122");
    await back.env.advance(DEBOUNCE_MS);
    expect(back.server.schedules.records.get(id)).toMatchObject({ name: "Schedule 1" });
  });
});
