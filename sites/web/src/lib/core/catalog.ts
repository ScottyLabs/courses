/* oxlint-disable typescript/consistent-return, unicorn/no-array-sort -- Courseweave's code, ported unchanged (decision 14) so fixes carry over between the two codebases */
// Requisite graph helpers over the course catalog. Loading the catalog documents is lib/data's job.
import type { CatalogIndex, Requisite } from "./catalog-types.ts";
import { isRequired, requisiteCourses } from "./requisite-format.ts";

export {
  formatRequisite,
  isRequired,
  requisiteCourses,
  requisiteSize,
} from "./requisite-format.ts";

export type Loadable<T> =
  | { status: "loading" }
  | { status: "error"; error: string }
  | { status: "ready"; value: T };

export interface Unlock {
  /** The course that lists the requirement. */
  id: string;
  /** prereq: appears in its prerequisites; coreq: listed as a corequisite (and not a prerequisite). */
  via: "prereq" | "coreq";
  /** True when every way to satisfy its prerequisites needs this course (always false for coreqs). */
  required: boolean;
}

const unlockCache = new WeakMap<CatalogIndex, Map<string, Unlock[]>>();

/** Reverse requisite index: course id -> courses whose prerequisites or corequisites mention it. */
export function buildUnlocks(index: CatalogIndex): Map<string, Unlock[]> {
  const cached = unlockCache.get(index);
  if (cached) return cached;
  const out = new Map<string, Unlock[]>();
  const add = (from: string, u: Unlock) => {
    const list = out.get(from);
    if (list) list.push(u);
    else out.set(from, [u]);
  };
  for (const entry of Object.values(index.courses)) {
    const pre = requisiteCourses(entry.prereqs);
    for (const id of pre)
      if (id !== entry.id)
        add(id, { id: entry.id, via: "prereq", required: isRequired(entry.prereqs, id) });
    for (const id of entry.coreqs)
      if (id !== entry.id && !pre.includes(id))
        add(id, { id: entry.id, via: "coreq", required: false });
  }
  for (const list of out.values()) list.sort((a, b) => a.id.localeCompare(b.id));
  unlockCache.set(index, out);
  return out;
}

/**
 * Length of the longest prerequisite chain below a course (0 = no prerequisites).
 * Takes the easiest alternative at each "or", so it measures the shortest path to eligibility.
 */
export function prerequisiteDepth(
  index: CatalogIndex,
  id: string,
  memo = new Map<string, number>(),
  visiting = new Set<string>(),
): number {
  const known = memo.get(id);
  if (known !== undefined) return known;
  if (visiting.has(id)) return 0;
  visiting.add(id);
  const depthOf = (r: Requisite): number => {
    switch (r.type) {
      case "course":
        return 1 + prerequisiteDepth(index, r.id, memo, visiting);
      case "text":
        return 0;
      case "all":
        return Math.max(...r.items.map(depthOf));
      case "any":
        return Math.min(...r.items.map(depthOf));
      case "atLeast": {
        const sorted = r.items.map(depthOf).sort((a, b) => a - b);
        return sorted[Math.min(r.n, sorted.length) - 1] ?? 0;
      }
    }
  };
  const req = index.courses[id]?.prereqs;
  const depth = req ? depthOf(req) : 0;
  visiting.delete(id);
  memo.set(id, depth);
  return depth;
}
