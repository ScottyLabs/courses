/* oxlint-disable typescript/no-floating-promises -- Courseweave's tests, ported unchanged (decision 14); node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../../testing.ts";
import { onboardingDue, type OnboardingInputs } from "./onboarding.ts";

const base: OnboardingInputs = {
  session: "signed-out",
  loaded: false,
  recordEmpty: true,
  asked: false,
};
const due = (patch: Partial<OnboardingInputs>) => onboardingDue({ ...base, ...patch });

describe("onboardingDue", () => {
  it("asks a new visitor with an empty record once", () => {
    expect(due({})).toBe(true);
    expect(due({ asked: true })).toBe(false);
    expect(due({ session: "unavailable" })).toBe(true);
  });

  it("never asks while the session is unknown or when a record exists", () => {
    expect(due({ session: "loading" })).toBe(false);
    expect(due({ recordEmpty: false })).toBe(false);
    expect(due({ session: "signed-in", loaded: true, recordEmpty: false })).toBe(false);
  });

  it("asks an account once its data has loaded, by the account's own answer", () => {
    const signedIn = { session: "signed-in" } as const;
    expect(due({ ...signedIn, loaded: false })).toBe(false);
    expect(due({ ...signedIn, loaded: true })).toBe(true);
    expect(due({ ...signedIn, loaded: true, asked: true })).toBe(false);
  });
});
