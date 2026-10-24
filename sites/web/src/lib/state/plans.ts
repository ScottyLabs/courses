// Degree plans (`/api/plans`): Courseweave's plan store, with each plan its own server record. The active plan
// lives in preferences. Edits that may land in a synced schedule are in link-actions.ts.
import type { Api } from "../api/endpoints.ts";
import { ApiError } from "../api/errors.ts";
import type { PlanRecord } from "../api/types.ts";
import type { ProgramSelection } from "../core/plan-types.ts";
import {
  applyPlanEdit,
  duplicatePlan,
  latestPlan,
  newPlan,
  planEdits,
  planList,
  type NewPlanOptions,
  type Plan,
  type PlanCourse,
  type PlanEdit,
  type SnapshotCourse,
} from "../core/state/plan.ts";
import { uid } from "../core/state/store.ts";
import { mergeNewer, normalizePlan } from "../core/sync-merge.ts";
import { CollectionStore } from "./collection.ts";
import type { StoreDeps } from "./drafts.ts";
import { isObject } from "./env.ts";
import { distinctName } from "./names.ts";
import type { PreferencesStore } from "./preferences.ts";
import type { Stored } from "./sync.ts";

function normalize(raw: unknown): Plan | null {
  return isObject(raw) && typeof raw.id === "string" ? normalizePlan(raw.id, raw) : null;
}

function fromApi(r: PlanRecord): Stored<Plan> {
  const value = normalize(r);
  if (value === null)
    throw new ApiError(200, "unavailable", "The server sent a plan this website can't read.");
  return { value, revision: r.revision };
}

export class PlanStore {
  readonly store: CollectionStore<Plan>;
  readonly #prefs: PreferencesStore;
  readonly #now: () => number;

  constructor(api: Api, prefs: PreferencesStore, deps: StoreDeps) {
    this.#prefs = prefs;
    this.#now = () => deps.env.now();
    this.store = new CollectionStore<Plan>(
      {
        name: "plans",
        list: async () => (await api.plans.list()).map((r) => fromApi(r)),
        create: async (p) =>
          fromApi(
            await api.plans.create({
              id: p.id,
              name: p.name,
              programs: p.programs,
              startTerm: p.startTerm,
              terms: p.terms,
            }),
          ),
        update: async (p, revision) =>
          fromApi(
            await api.plans.update(p.id, {
              name: p.name,
              programs: p.programs,
              startTerm: p.startTerm,
              terms: p.terms,
              revision,
            }),
          ),
        remove: (id) => api.plans.remove(id),
        normalize,
        merge: (local, remote) => mergeNewer(local, remote),
        // A plan drafted signed out keeps its courses, under a name none of the account's plans has.
        adoptDraft: (draft, existing) => {
          const name = distinctName(
            draft.name,
            existing.map((p) => p.name),
            "Plan",
          );
          return { value: name === draft.name ? draft : { ...draft, name }, save: true };
        },
      },
      deps,
    );
  }

  // ---- Reading ----

  get items(): Record<string, Plan> {
    return this.store.items;
  }

  get(id: string): Plan | undefined {
    return this.store.items[id];
  }

  /** Plans, oldest first (tab order). */
  get list(): Plan[] {
    return planList(Object.values(this.store.items));
  }

  /** The active plan's id: the one chosen, else the most recently edited plan, else null. */
  get activeId(): string | null {
    const chosen = this.#prefs.activePlanId;
    if (chosen !== null && this.store.items[chosen] !== undefined) return chosen;
    return latestPlan(Object.values(this.store.items))?.id ?? null;
  }

  get active(): Plan | null {
    const id = this.activeId;
    return id === null ? null : (this.store.items[id] ?? null);
  }

  // ---- Plans ----

  /** Creates a plan and makes it active. `terms` lists the semesters (default: eight from `startTerm`). */
  createPlan(options: NewPlanOptions = {}): string {
    const plan = newPlan(uid(), options, Object.values(this.store.items), this.#now());
    this.store.create(plan);
    this.#prefs.setActivePlan(plan.id);
    return plan.id;
  }

  /** A copy named "<name> copy", made active. Returns its id (or `id` when there's no such plan). */
  duplicatePlan(id: string): string {
    const p = this.store.items[id];
    if (p === undefined) return id;
    const copy = duplicatePlan(p, uid(), this.#now());
    this.store.create(copy);
    this.#prefs.setActivePlan(copy.id);
    return copy.id;
  }

  deletePlan(id: string): void {
    if (this.store.items[id] === undefined) return;
    this.store.delete(id);
    if (this.#prefs.activePlanId === id)
      this.#prefs.setActivePlan(latestPlan(Object.values(this.store.items))?.id ?? null);
  }

  setActivePlan(id: string | null): void {
    this.#prefs.setActivePlan(id);
  }

  // ---- Edits ----

  #edit(id: string, edit: PlanEdit): void {
    const now = this.#now();
    this.store.edit(id, (p) => applyPlanEdit(p, edit, now));
  }

  renamePlan(id: string, name: string): void {
    this.#edit(id, planEdits.rename(name));
  }

  setPrograms(planId: string, programs: ProgramSelection[]): void {
    this.#edit(planId, planEdits.setPrograms(programs));
  }

  /** Moves the plan's start; adds default semesters from it and drops empty semesters before it. */
  setStartTerm(planId: string, startTerm: string): void {
    this.#edit(planId, planEdits.setStartTerm(startTerm));
  }

  addTerm(planId: string, term: string): void {
    this.#edit(planId, planEdits.addTerm(term));
  }

  /** Removes a semester and the courses planned in it. */
  removeTerm(planId: string, term: string): void {
    this.#edit(planId, planEdits.removeTerm(term));
  }

  /** Adds a course to a semester (created if needed). Returns the plan-course id. */
  addCourse(
    planId: string,
    term: string,
    courseId: string,
    options: { passFail?: boolean; units?: number } = {},
  ): string {
    const id = uid();
    this.#edit(planId, planEdits.addCourse(id, term, courseId, options));
    return id;
  }

  removeCourse(planId: string, planCourseId: string): void {
    this.#edit(planId, planEdits.removeCourse(planCourseId));
  }

  /** Moves a course to another semester (created if needed), at `index` or the end. */
  moveCourse(planId: string, planCourseId: string, toTerm: string, index?: number): void {
    this.#edit(planId, planEdits.moveCourse(planCourseId, toTerm, index));
  }

  updateCourse(
    planId: string,
    planCourseId: string,
    patch: Partial<Pick<PlanCourse, "passFail" | "units">>,
  ): void {
    this.#edit(planId, planEdits.updateCourse(planCourseId, patch));
  }

  /** Replaces a semester's courses, e.g. with a schedule from the schedule planner. */
  setTermCourses(
    planId: string,
    term: string,
    courses: { courseId: string; passFail?: boolean }[],
  ): void {
    this.#edit(planId, planEdits.setTermCourses(term, courses));
  }

  /** Syncs a semester with a schedule; `snapshot` is kept as the fallback should the schedule be deleted. */
  linkTerm(planId: string, term: string, scheduleId: string, snapshot: SnapshotCourse[]): void {
    this.#edit(planId, planEdits.linkTerm(term, scheduleId, snapshot));
  }

  /** Stops syncing; the semester keeps `courses` (normally the schedule's current courses) as planned courses. */
  unlinkTerm(planId: string, term: string, courses?: SnapshotCourse[]): void {
    this.#edit(planId, planEdits.unlinkTerm(term, courses));
  }

  /** Units override for a course in a semester (variable-unit courses), creating its snapshot row if needed. */
  setCourseUnits(planId: string, term: string, courseId: string, units: number | undefined): void {
    this.#edit(planId, planEdits.setCourseUnits(term, courseId, units));
  }

  /** Drops a course from a semester's own list by course id (keeps a synced semester's snapshot current). */
  removeCourseFromTerm(planId: string, term: string, courseId: string): void {
    this.#edit(planId, planEdits.removeCourseFromTerm(term, courseId));
  }
}
