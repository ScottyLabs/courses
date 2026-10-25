// The profile's schedule previews (SchedulesSection's Preview view): the semesters the schedules are grouped under,
// each schedule's meetings for its miniature week, and the card's class count.
import type { Term } from "../../core/data.ts";
import { partBlocks, plural } from "../../core/planner/format.ts";
import { resolveEntries } from "../../core/state/schedule-view.ts";
import type { Schedule } from "../../core/state/store.ts";
import type { CalendarBlock } from "../../core/week-grid.ts";

/** Schedules grouped by semester, keeping the order they come in (newest semester first on the profile). */
export function bySemester<S extends { termId: string }>(
  schedules: readonly S[],
): { termId: string; schedules: S[] }[] {
  const groups = new Map<string, S[]>();
  for (const schedule of schedules) {
    const group = groups.get(schedule.termId);
    if (group === undefined) groups.set(schedule.termId, [schedule]);
    else group.push(schedule);
  }
  return [...groups].map(([termId, list]) => ({ termId, schedules: list }));
}

/** The meetings of the schedule's chosen sections. Hidden courses and courses without a set time are left out. */
export function previewBlocks(schedule: Schedule, term: Term): CalendarBlock[] {
  return resolveEntries(schedule, term)
    .filter((resolved) => !resolved.entry.hidden)
    .flatMap((resolved) =>
      resolved.parts.flatMap((part) => partBlocks(resolved.course, part, resolved.entry.color)),
    );
}

/** "3 classes", "1 class" or "No classes yet": the courses that aren't hidden, as the details cards list them. */
export function classCount(schedule: Schedule): string {
  const count = schedule.entries.filter((entry) => !entry.hidden).length;
  return count === 0 ? "No classes yet" : plural(count, "class", "classes");
}
