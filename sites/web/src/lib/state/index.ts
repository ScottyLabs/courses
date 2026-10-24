// The student's own data and device preferences (decision 17). The instances are what pages use; the classes are
// for tests. See docs/src/frontend.md.
export {
  device,
  links,
  personal,
  plans,
  preferences,
  record,
  saved,
  schedules,
} from "./instances.svelte.ts";
export { Personal } from "./personal.ts";
export type { PersonalDeps, SignOutResult } from "./personal.ts";
export { ScheduleStore } from "./schedules.ts";
export type { ScheduleItem } from "./schedules.ts";
export { PlanStore } from "./plans.ts";
export { RecordStore } from "./record.ts";
export { SavedStore } from "./saved.ts";
export {
  PreferencesStore,
  DEFAULT_PREFERENCES,
  mergePreferences,
  normalizePreferences,
} from "./preferences.ts";
export { DevicePrefs, DEFAULT_DEVICE } from "./device.ts";
export type { DeviceState } from "./device.ts";
export { linkActions } from "./link-actions.ts";
export type { LinkActions } from "./link-actions.ts";
export { CollectionStore } from "./collection.ts";
export type { StoreDeps, StoreMode } from "./drafts.ts";
export { DocumentStore } from "./document.ts";
export { worstStatus } from "./sync.ts";
export type { SaveStatus } from "./sync.ts";
export { browserEnv } from "./env.ts";
export type { StoreEnv } from "./env.ts";
export { ALL_SEMESTERS, ALL_SEMESTER_PAGES, isAllSemesters, resolveTerm } from "./term-scope.ts";
export { lastOrigin, planOrigin, trackOrigin } from "./nav-origin.ts";
export {
  buildScheduleView,
  fittingSections,
  fittingSelections,
  partsConflict,
  resolveEntries,
  resolveEntry,
  unitTotals,
} from "../core/state/schedule-view.ts";
export type { ResolvedEntry, ScheduleView, UnitTotals } from "../core/state/schedule-view.ts";
export { shellSession } from "./shell-session.ts";
