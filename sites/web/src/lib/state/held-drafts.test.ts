/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../testing.ts";
import { DEBOUNCE_MS, harness } from "./test-env.ts";

describe("drafts held on signing in", () => {
  it("holds signed-out drafts until the student adds or discards them, across reloads and sign-outs", async () => {
    const h = await harness();
    const id = h.personal.schedules.ensureSchedule("S27");
    h.personal.schedules.addCourse(id, "15122");
    // An empty default schedule for another semester isn't worth asking about.
    h.personal.schedules.ensureSchedule("F26");
    h.personal.saved.add("15213");

    h.server.signedIn = true;
    const back = await h.reload();
    expect(back.personal.heldDrafts).toEqual({ schedules: 1, plans: 0, record: false, saved: 1 });
    expect(back.personal.schedules.forTerm("S27")).toEqual([]);
    expect(back.personal.schedules.forTerm("F26")).toHaveLength(1);
    expect(back.personal.saved.ids).toEqual([]);

    // A reload asks again.
    const again = await back.reload();
    expect(again.personal.heldDrafts).toEqual({ schedules: 1, plans: 0, record: false, saved: 1 });

    // Signing out (say, into the wrong account) gives the tab its drafts back.
    expect(await again.personal.signOut()).toBe("done");
    expect(again.personal.schedules.get(id)?.entries).toHaveLength(1);
    expect(again.personal.saved.ids).toEqual(["15213"]);

    // Discarding forgets them; the account never had them.
    again.server.signedIn = true;
    const other = await again.reload();
    expect(other.personal.heldDrafts).not.toBeNull();
    other.personal.discardDrafts();
    expect(other.personal.heldDrafts).toBeNull();
    await other.env.advance(10 * DEBOUNCE_MS);
    expect(other.server.schedules.records.has(id)).toBe(false);
    expect(other.server.saved.courses.size).toBe(0);
    const last = await other.reload();
    expect(last.personal.heldDrafts).toBeNull();
    expect(last.personal.schedules.get(id)).toBeUndefined();
  });
});
