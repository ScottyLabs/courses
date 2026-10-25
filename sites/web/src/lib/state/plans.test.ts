/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../testing.ts";
import { DEBOUNCE_MS, harness } from "./test-env.ts";

describe("plan programs", () => {
  it("saves a program's concentrations to the account, keeps them over a reload and clears them", async () => {
    const h = await harness({ signedIn: true });
    const id = h.personal.plans.createPlan({
      startTerm: "F25",
      programs: [{ catalogId: 1, auditId: 11 }],
    });
    h.personal.plans.setPrograms(id, [{ catalogId: 1, auditId: 11, concentrations: [7, 8] }]);
    expect(h.personal.plans.get(id)?.programs).toEqual([
      { catalogId: 1, auditId: 11, concentrations: [7, 8] },
    ]);
    await h.env.advance(DEBOUNCE_MS);
    expect(h.server.plans.records.get(id)?.programs).toEqual([
      { catalogId: 1, auditId: 11, concentrations: [7, 8] },
    ]);

    const back = await h.reload();
    expect(back.personal.plans.get(id)?.programs).toEqual([
      { catalogId: 1, auditId: 11, concentrations: [7, 8] },
    ]);

    back.personal.plans.setPrograms(id, [{ catalogId: 1, auditId: 11, concentrations: [] }]);
    await back.env.advance(DEBOUNCE_MS);
    const again = await back.reload();
    expect(again.personal.plans.get(id)?.programs[0]?.concentrations ?? []).toEqual([]);
  });
});
