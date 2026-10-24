// How Courses Taken lists the record's courses: cards or a list, and the department, level and semester filters. The
// filters sit in the profile's title band (TakenFilters) and the list further down (TakenCourses), so they share this
// state. It lasts for the tab, like the explorer's filters.
import { ALL_FILTERS, takenEntries, type ListFilters } from "$lib/core/record/taken-list";
import { record } from "$lib/state";

/** `view` is null until the student picks one here; until then it's the device's stored choice (`storedView`). */
export const takenList = $state<{ view: "cards" | "list" | null; filters: ListFilters }>({
  view: null,
  filters: { ...ALL_FILTERS },
});

/** Whether any filter is on. */
export function takenFiltered(): boolean {
  const f = takenList.filters;
  return f.department !== "" || f.level !== "" || f.semester !== "";
}

export function clearTakenFilters(): void {
  takenList.filters = { ...ALL_FILTERS };
}

/** Whether the record lists any CMU courses, so the filters have something to filter. */
export function hasTakenCourses(): boolean {
  return takenEntries(record.entries).length > 0;
}
