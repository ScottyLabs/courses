// First-run prompt to add the academic record (past courses, AP/IB/transfer credit). Asked once per account (the
// answer is in the account's preferences) and, signed out, once per device; never when a record already exists.

/** What the account (or, signed out, this device) remembers about the prompt. */
export interface OnboardingState {
  /** The student added their record or skipped. */
  asked: boolean;
  /** The "add your record" nudges on other pages were closed. */
  promptHidden: boolean;
}

export const EMPTY_ONBOARDING: OnboardingState = { asked: false, promptHidden: false };

export type SessionStatus = "loading" | "signed-in" | "signed-out" | "unavailable";

export interface OnboardingInputs {
  session: SessionStatus;
  /** Signed in: the account's record and preferences have loaded. */
  loaded: boolean;
  recordEmpty: boolean;
  /** From the account's preferences when signed in, from this device otherwise. */
  asked: boolean;
}

/** Whether to ask now. Signed in, it waits for the account's data so a record saved elsewhere isn't asked for again. */
export function onboardingDue(i: OnboardingInputs): boolean {
  if (!i.recordEmpty || i.session === "loading") return false;
  if (i.session === "signed-in") return i.loaded && !i.asked;
  return !i.asked;
}
