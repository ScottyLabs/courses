// The student's requisite status for course components: what they've taken (record), plan to take before a
// semester (degree plan, other semesters' schedules) and take alongside (that semester's schedule).
import { requisiteContext } from "$lib/core/record/context";
import { statusFnFor } from "$lib/core/record/check-summary";
import type { RequisiteContext } from "$lib/core/requisites";
import { plans, preferences, record, schedules } from "$lib/state";
import { catalogIndex, termData } from "./data.svelte";

/** How a requisite course stands, as the status chips show it. */
export type ChipStatus = "satisfied" | "planned" | "in-progress" | "unsatisfied";

export type ChipStatusFn = (courseId: string) => ChipStatus | undefined;

const CHIP_STATUS = { met: "satisfied", planned: "planned", missing: "unsatisfied" } as const;

function chipStatus(fn: (id: string) => "met" | "planned" | "missing" | undefined): ChipStatusFn {
  return (id) => {
    const status = fn(id);
    return status === undefined ? status : CHIP_STATUS[status];
  };
}

// Plain memoized getters rather than $derived fields: these objects are often created inside a reaction (the first
// card to ask), and the getters read only reactive sources made elsewhere, so every caller stays tracked.
class TermRequisites {
  readonly termId: string;
  #inputs: unknown[] = [];
  #ctx: RequisiteContext | null = null;
  #prereq: ChipStatusFn | null = null;
  #coreq: ChipStatusFn | null = null;

  constructor(termId: string) {
    this.termId = termId;
  }

  /** Checked against the record, the active degree plan and the active schedules; null while loading. */
  get ctx(): RequisiteContext | null {
    const term = termData(this.termId).value;
    const index = catalogIndex().value;
    const items = schedules.items;
    const inputs = [term, index, items, record.data, plans.active, preferences.activeSchedule];
    if (inputs.every((input, i) => input === this.#inputs[i])) return this.#ctx;
    this.#inputs = inputs;
    this.#prereq = null;
    this.#coreq = null;
    if (term === undefined || index === undefined) {
      this.#ctx = null;
      return null;
    }
    const activeSchedule: Record<string, string> = {};
    for (const item of Object.values(items)) {
      if (activeSchedule[item.termId] !== undefined) continue;
      const id = schedules.activeId(item.termId);
      if (id !== null) activeSchedule[item.termId] = id;
    }
    this.#ctx = requisiteContext({
      term,
      record: record.data,
      catalog: index,
      plan: plans.active,
      schedules: items,
      activeSchedule,
    });
    return this.#ctx;
  }

  /** Chip status for a course as a prerequisite; null while loading. */
  get prereqStatus(): ChipStatusFn | null {
    const ctx = this.ctx;
    if (ctx === null) return null;
    this.#prereq ??= chipStatus(statusFnFor(ctx, false));
    return this.#prereq;
  }

  /** Chip status for a course as a corequisite (same-semester courses count); null while loading. */
  get coreqStatus(): ChipStatusFn | null {
    const ctx = this.ctx;
    if (ctx === null) return null;
    this.#coreq ??= chipStatus(statusFnFor(ctx, true));
    return this.#coreq;
  }
}

const requisiteCache = new Map<string, TermRequisites>();

/**
 * The student's requisite context for a semester: what they've taken (record), plan to take before it (degree plan,
 * other semesters' schedules) and take alongside (that semester's schedule). Shared by every component.
 */
export function termRequisites(termId: string): TermRequisites {
  let entry = requisiteCache.get(termId);
  if (entry === undefined) {
    entry = new TermRequisites(termId);
    requisiteCache.set(termId, entry);
  }
  return entry;
}

export type { TermRequisites };
