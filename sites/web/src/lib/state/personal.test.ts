/* oxlint-disable typescript/no-floating-promises, eslint/max-lines-per-function, typescript/no-unsafe-type-assertion, eslint/no-unsafe-optional-chaining -- node:test awaits describe and it itself; a describe groups many cases; fixtures are cast to their shapes */
import { describe, it } from "node:test";
import { expect } from "../testing.ts";
import { DEBOUNCE_MS, harness, settle } from "./test-env.ts";

const writes = (log: string[]) => log.filter((l) => !l.startsWith("GET"));

describe("signed out", () => {
  it("keeps drafts in sessionStorage, and a reload brings them back", async () => {
    const h = await harness();
    expect(h.session.status).toBe("signed-out");
    expect(h.personal.mode).toBe("draft");
    const id = h.personal.schedules.createSchedule("S27");
    h.personal.schedules.addCourse(id, "15122");
    h.personal.saved.add("15213");
    h.personal.record.addEntry({ courseId: "15122", source: "taken", term: "F25", grade: "A" });
    h.personal.preferences.setPlanLayout("table");
    expect(h.env.store.keys().toSorted()).toEqual([
      "cmucourses:draft:preferences",
      "cmucourses:draft:record",
      "cmucourses:draft:saved",
      "cmucourses:draft:schedules",
    ]);
    // Nothing personal goes to the server signed out.
    expect(writes(h.server.log)).toEqual([]);

    const again = await h.reload();
    expect(again.personal.schedules.get(id)?.entries.map((e) => e.courseId)).toEqual(["15122"]);
    expect(again.personal.schedules.activeId("S27")).toBe(id);
    expect(again.personal.saved.ids).toEqual(["15213"]);
    expect(again.personal.record.entries).toHaveLength(1);
    expect(again.personal.preferences.planLayout).toBe("table");
  });

  it("makes a semester's first schedule without saving it until it changes", async () => {
    const h = await harness({ signedIn: true });
    const id = h.personal.schedules.ensureSchedule("S27");
    expect(h.personal.schedules.ensureSchedule("S27")).toBe(id);
    await h.env.advance(10 * DEBOUNCE_MS);
    expect(h.server.schedules.records.size).toBe(0);
    h.personal.schedules.addCourse(id, "15122");
    await h.env.advance(DEBOUNCE_MS);
    expect(h.server.schedules.records.get(id)?.entries).toHaveLength(1);
  });
});

describe("signing in", () => {
  it("saves the signed-out drafts to the account once the student adds them, and clears them", async () => {
    const h = await harness();
    const id = h.personal.schedules.createSchedule("S27", "Draft schedule");
    h.personal.schedules.addCourse(id, "15122");
    h.personal.saved.add("15213");
    h.personal.record.addEntry({
      courseId: "21127",
      source: "exam",
      units: 10,
      exam: { id: "ap", score: "5" },
    });
    h.personal.plans.createPlan({ startTerm: "F26", name: "Draft plan" });

    // The account already has a record entry and a saved course.
    h.server.record.value = {
      ...h.server.record.value,
      entries: [
        { id: "srv", courseId: "15122", source: "taken", term: "F25", grade: "A", addedAt: 1 },
      ],
      revision: 3,
    };
    h.server.saved.courses.set("76101", 1);

    h.session.signIn();
    expect(h.navigations).toEqual(["/auth/login?returnTo=%2Fschedules%3Fterm%3DS27"]);
    // Keycloak signs in; the page comes back and loads.
    h.server.signedIn = true;
    const back = await h.reload();
    expect(back.personal.mode).toBe("account");
    // Nothing joins the account until the student says so.
    expect(back.personal.heldDrafts).toEqual({ schedules: 1, plans: 1, record: true, saved: 1 });
    expect(back.personal.onboardingDue).toBe(false);
    await back.env.advance(DEBOUNCE_MS);
    expect(back.server.schedules.records.size).toBe(0);
    expect(back.server.plans.records.size).toBe(0);
    expect([...back.server.saved.courses.keys()]).toEqual(["76101"]);

    back.personal.acceptDrafts();
    expect(back.personal.heldDrafts).toBeNull();
    await back.env.advance(DEBOUNCE_MS);

    expect(back.server.schedules.records.get(id)).toMatchObject({
      name: "Draft schedule",
      revision: 1,
    });
    expect([...back.server.plans.records.values()].map((p) => p.name)).toEqual(["Draft plan"]);
    expect([...back.server.saved.courses.keys()].toSorted()).toEqual(["15213", "76101"]);
    const entries = back.server.record.value.entries as { courseId: string }[];
    expect(entries.map((e) => e.courseId).toSorted()).toEqual(["15122", "21127"]);
    expect(back.server.preferences.value.activeSchedule).toEqual({ S27: id });
    expect(back.personal.status).toBe("saved");
    // The API brought the student back to the page they were on, so there's nowhere else to go.
    expect(back.session.consumeReturnPath()).toBeNull();
    expect(back.env.store.keys()).toEqual([]);
  });

  it("loads the account's data after signing in", async () => {
    const h = await harness({ signedIn: true });
    h.server.schedules.records.set("s1", {
      id: "s1",
      termId: "S27",
      name: "Mine",
      entries: [],
      visibility: "private",
      shareId: null,
      revision: 4,
      createdAt: 1,
      updatedAt: 1,
    });
    const back = await h.reload();
    expect(back.personal.schedules.forTerm("S27").map((s) => s.name)).toEqual(["Mine"]);
    expect(back.personal.schedules.activeId("S27")).toBe("s1");
  });
});

describe("saving to the account", () => {
  it("debounces edits into one write per record", async () => {
    const h = await harness({ signedIn: true });
    const id = h.personal.schedules.createSchedule("S27");
    for (const c of ["15122", "15150", "21127", "76101"]) h.personal.schedules.addCourse(id, c);
    h.personal.schedules.renameSchedule(id, "Spring");
    expect(h.personal.status).toBe("saving");
    await h.env.advance(DEBOUNCE_MS);
    expect(writes(h.server.log).filter((l) => l.includes("/api/schedules"))).toEqual([
      "POST /api/schedules",
    ]);
    expect(h.server.schedules.records.get(id)).toMatchObject({ name: "Spring", revision: 1 });
    h.personal.schedules.removeCourse(id, "76101");
    h.personal.schedules.setPassFail(id, "21127", true);
    await h.env.advance(DEBOUNCE_MS);
    expect(writes(h.server.log).filter((l) => l.includes("/api/schedules"))).toEqual([
      "POST /api/schedules",
      `PUT /api/schedules/${id}`,
    ]);
    expect(h.server.schedules.records.get(id)?.revision).toBe(2);
    expect(h.personal.status).toBe("saved");
  });

  it("reapplies an edit onto another device's newer schedule (409)", async () => {
    const h = await harness({ signedIn: true });
    const id = h.personal.schedules.createSchedule("S27");
    h.personal.schedules.addCourse(id, "15122");
    await h.env.advance(DEBOUNCE_MS);
    // Another device adds a course and renames the schedule.
    const stored = h.server.schedules.records.get(id);
    if (stored === undefined) throw new Error("not created");
    const theirs = [
      ...(stored.entries as object[]),
      { courseId: "21127", section: null, child: null, passFail: false, color: 1, hidden: false },
    ];
    h.server.schedules.records.set(id, {
      ...stored,
      name: "Theirs",
      entries: theirs,
      revision: stored.revision + 1,
      updatedAt: h.env.now() + 5000,
    });

    h.personal.schedules.addCourse(id, "15150");
    await h.env.advance(DEBOUNCE_MS);
    const final = h.server.schedules.records.get(id);
    expect(final?.name).toBe("Theirs");
    expect((final?.entries as { courseId: string }[]).map((e) => e.courseId)).toEqual([
      "15122",
      "21127",
      "15150",
    ]);
    expect(h.personal.schedules.get(id)?.name).toBe("Theirs");
    expect(h.personal.status).toBe("saved");
  });

  it("merges the record on a conflict without losing either side", async () => {
    const h = await harness({ signedIn: true });
    h.personal.record.addEntry({ courseId: "15122", source: "taken", term: "F25", grade: "A" });
    await h.env.advance(DEBOUNCE_MS);
    h.server.record.value = {
      ...h.server.record.value,
      entries: [
        ...(h.server.record.value.entries as object[]),
        { id: "other", courseId: "21127", source: "taken", term: "F25", addedAt: 5 },
      ],
      revision: h.server.record.value.revision + 1,
    };
    h.personal.record.setProfile({ college: "SCS" });
    await h.env.advance(DEBOUNCE_MS);
    expect(
      (h.server.record.value.entries as { courseId: string }[]).map((e) => e.courseId),
    ).toEqual(["15122", "21127"]);
    expect(h.server.record.value.profile).toEqual({ college: "SCS" });
  });

  it("keeps unsaved changes through a lost connection and a reload", async () => {
    const h = await harness({ signedIn: true });
    const id = h.personal.schedules.createSchedule("S27");
    await h.env.advance(DEBOUNCE_MS);
    h.server.fail("PUT", /\/api\/schedules\//u, "network");
    h.personal.schedules.addCourse(id, "15122");
    await h.env.advance(DEBOUNCE_MS);
    expect(h.personal.status).toBe("offline");
    expect(h.personal.unsaved).toBe(true);
    expect(h.env.store.keys()).toEqual(["cmucourses:pending:schedules"]);

    const back = await h.reload();
    await back.env.advance(DEBOUNCE_MS);
    expect(
      (back.server.schedules.records.get(id)?.entries as { courseId: string }[]).map(
        (e) => e.courseId,
      ),
    ).toEqual(["15122"]);
    expect(back.personal.status).toBe("saved");
    expect(back.env.store.keys()).toEqual([]);
  });

  it("retries when the browser comes back online", async () => {
    const h = await harness({ signedIn: true });
    h.env.isOnline = false;
    h.personal.saved.add("15122");
    const id = h.personal.schedules.createSchedule("S27");
    await h.env.advance(DEBOUNCE_MS);
    expect(h.personal.status).toBe("offline");
    await h.env.goOnline();
    expect(h.server.saved.courses.has("15122")).toBe(true);
    expect(h.server.schedules.records.has(id)).toBe(true);
    expect(h.personal.status).toBe("saved");
  });

  it("deletes on the server, even a schedule still being created", async () => {
    const h = await harness({ signedIn: true });
    const keep = h.personal.schedules.createSchedule("S27", "Keep");
    const gone = h.personal.schedules.createSchedule("S27", "Gone");
    await h.env.advance(DEBOUNCE_MS);
    h.personal.schedules.deleteSchedule(gone);
    await settle();
    expect([...h.server.schedules.records.keys()]).toEqual([keep]);
    expect(h.personal.schedules.activeId("S27")).toBe(keep);
  });
});

describe("the session ending", () => {
  it("keeps the change on screen and in storage when the session expires", async () => {
    const h = await harness({ signedIn: true });
    const id = h.personal.schedules.createSchedule("S27");
    await h.env.advance(DEBOUNCE_MS);
    h.server.signedIn = false;
    h.personal.schedules.addCourse(id, "15122");
    await h.env.advance(DEBOUNCE_MS);
    expect(h.session.status).toBe("signed-out");
    expect(h.personal.status).toBe("error");
    expect(h.personal.error).toContain("Sign in again");
    expect(h.personal.schedules.get(id)?.entries).toHaveLength(1);
    expect(h.env.store.keys()).toContain("cmucourses:pending:schedules");

    // Signing in again saves it.
    h.server.signedIn = true;
    const back = await h.reload();
    await back.env.advance(DEBOUNCE_MS);
    expect(back.server.schedules.records.get(id)?.entries).toHaveLength(1);
  });

  it("signs out after saving, and forgets the account's data", async () => {
    const h = await harness({ signedIn: true });
    h.personal.schedules.createSchedule("S27");
    h.personal.saved.add("15122");
    expect(await h.personal.signOut()).toBe("done");
    expect(h.server.schedules.records.size).toBe(1);
    expect(h.server.signedIn).toBe(false);
    expect(h.personal.mode).toBe("draft");
    expect(h.personal.schedules.forTerm("S27")).toEqual([]);
    expect(h.personal.saved.ids).toEqual([]);
    expect(h.env.store.keys()).toEqual([]);
  });

  it("won't sign out with changes the server couldn't take, unless forced", async () => {
    const h = await harness({ signedIn: true });
    h.env.isOnline = false;
    h.personal.saved.add("15122");
    expect(await h.personal.signOut()).toBe("unsaved");
    h.env.isOnline = true;
    expect(await h.personal.signOut(true)).toBe("done");
  });
});

describe("onboarding", () => {
  it("asks once the account has loaded and nothing is recorded", async () => {
    const h = await harness({ signedIn: true });
    expect(h.personal.onboardingDue).toBe(true);
    h.personal.markOnboardingAsked();
    expect(h.personal.onboardingDue).toBe(false);
    await h.env.advance(DEBOUNCE_MS);
    expect(h.server.preferences.value.onboarding).toEqual({ asked: true, promptHidden: false });
  });
});
