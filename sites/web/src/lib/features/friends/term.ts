// The semester friends' courses and schedules default to: the one in session today, else the next to start, else the
// newest built one.
import type { TermIndexEntry } from "../../core/types.ts";

/** `today` is YYYY-MM-DD. Terms are newest first, as the catalog lists them. */
export function currentTerm(terms: readonly TermIndexEntry[], today: string): string | null {
  const now = terms.find((t) => t.classesBegin <= today && today <= t.classesEnd);
  if (now !== undefined) return now.id;
  const upcoming = terms.findLast((t) => t.classesBegin > today);
  return upcoming?.id ?? terms[0]?.id ?? null;
}

/** Today in Pittsburgh as YYYY-MM-DD. */
export function todayInPittsburgh(now = new Date()): string {
  return new Intl.DateTimeFormat("en-CA", { timeZone: "America/New_York" }).format(now);
}
