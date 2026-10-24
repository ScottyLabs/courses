// The semester a page shows is its `term` query parameter. "All semesters" (`term=all`) stands in for a semester on
// the course and instructor lists only; everything else needs a real one, and falls back to the last one visited
// (DevicePrefs.lastTerm), then the newest semester. From Courseweave's state/term-scope.ts.
import { ALL_TERMS } from "../core/routes.ts";
import type { TermIndexEntry } from "../core/types.ts";

/** The `term` value for the all-semesters view. */
export const ALL_SEMESTERS = ALL_TERMS;

/** Pages with an all-semesters view (route ids). */
export const ALL_SEMESTER_PAGES = ["/", "/instructors"];

/** Whether `term` is the all-semesters view. */
export function isAllSemesters(term: string | null | undefined): boolean {
  return term === ALL_SEMESTERS;
}

/**
 * The real semester to show: `requested` when it's built, else the last one visited when it's built, else the
 * newest built semester (terms are newest first). Null only when no semester is built.
 */
export function resolveTerm(
  requested: string | null | undefined,
  terms: TermIndexEntry[],
  lastTerm: string | null,
): string | null {
  const built = (id: string | null | undefined): id is string =>
    typeof id === "string" && terms.some((t) => t.id === id);
  if (built(requested)) return requested;
  if (built(lastTerm)) return lastTerm;
  return terms[0]?.id ?? null;
}

/** Options for a semester picker with an all-semesters view: every built semester, newest first, then "All semesters". */
export function semesterOptions(
  terms: readonly TermIndexEntry[],
): { value: string; label: string }[] {
  return [
    ...terms.map((term) => ({ value: term.id, label: term.name })),
    { value: ALL_SEMESTERS, label: "All semesters" },
  ];
}
