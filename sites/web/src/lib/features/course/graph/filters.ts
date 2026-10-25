// Filtering what a course unlocks on the requisite graph, and naming course-number prefixes as departments.
import type { Unlock } from "../../../core/catalog.ts";
import type { Department } from "../../../core/data.ts";
import { departmentName } from "../../../core/departments.ts";

export interface UnlockFilter {
  requiredOnly: boolean;
  offeredOnly: boolean;
  readyOnly: boolean;
  /** A course-number prefix, or "" for every department. */
  department: string;
}

export const NO_FILTER: UnlockFilter = {
  requiredOnly: false,
  offeredOnly: false,
  readyOnly: false,
  department: "",
};

/** The unlocks that pass the filter. `offered` is omitted where there's no semester; `ready` are takeable next. */
export function filterUnlocks(
  unlocks: readonly Unlock[],
  filter: UnlockFilter,
  offered: ((id: string) => boolean) | undefined,
  ready: ReadonlySet<string>,
): Unlock[] {
  return unlocks.filter(
    (u) =>
      (!filter.requiredOnly || u.required) &&
      (!filter.offeredOnly || offered === undefined || offered(u.id)) &&
      (!filter.readyOnly || ready.has(u.id)) &&
      (filter.department === "" || u.id.startsWith(filter.department)),
  );
}

/** Names a course-number prefix by the semester's departments, else the catalog's list. */
export function prefixNamer(departments: readonly Department[]): (prefix: string) => string {
  const names = new Map<string, string>();
  for (const d of departments)
    for (const prefix of d.prefixes) if (!names.has(prefix)) names.set(prefix, d.name);
  return (prefix) => names.get(prefix) ?? departmentName(prefix);
}

/** "All departments (81)", then each department with its count, the biggest first. */
export function departmentOptions(
  unlocks: readonly Unlock[],
  nameOf: (prefix: string) => string,
): { value: string; label: string }[] {
  const counts = new Map<string, number>();
  for (const u of unlocks) counts.set(u.id.slice(0, 2), (counts.get(u.id.slice(0, 2)) ?? 0) + 1);
  return [
    { value: "", label: `All departments (${unlocks.length})` },
    ...[...counts]
      .toSorted((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
      .map(([prefix, n]) => ({ value: prefix, label: `${nameOf(prefix)} (${n})` })),
  ];
}
