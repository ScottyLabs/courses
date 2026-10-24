// What the record's components look courses up in: the catalog index, the semester the student last browsed (for
// courses the catalog doesn't list), the offerings list (departments and levels) and FCE hours. Create it while a
// component initializes; the catalog shares each document between callers.
import type { CatalogIndex } from "$lib/core/catalog-types";
import type { Term } from "$lib/core/data";
import { currentTerm } from "$lib/core/degree/hooks";
import type { FceSummaryData } from "$lib/core/fce";
import type { Offerings } from "$lib/core/offerings";
import { courseLookup, termOptions, type CourseInfo } from "$lib/core/record/hooks";
import type { CourseFacts } from "$lib/core/record/taken-list";
import { termOrdinal } from "$lib/core/requisites";
import type { TermIndexEntry } from "$lib/core/types";
import { catalog, resource } from "$lib/data";
import { device, resolveTerm } from "$lib/state";

export interface RecordData {
  /** Every built semester, newest first (empty until loaded). */
  readonly terms: TermIndexEntry[];
  /** The semester used for lookups and course search: the last one visited, else the newest. */
  readonly term: Term | null;
  readonly index: CatalogIndex | null;
  readonly offerings: Offerings | null;
  /** FCE summaries; null signed out or when this catalog has none. */
  readonly fce: FceSummaryData | null;
  /** The semester in progress today. */
  readonly current: string;
  /** Title, code and units for any course. */
  lookup(courseId: string): CourseInfo;
  /** Department and level for the profile's filters. */
  facts(courseId: string): CourseFacts;
  /** FCE hours a week, or null without FCEs. */
  hours(courseId: string): number | null;
  /** Semester choices for "when did you take it", newest first. */
  termChoices(entryTerm?: string): string[];
}

export function recordData(): RecordData {
  const terms = resource(() => catalog.terms());
  const termId = $derived(
    terms.value === undefined ? null : resolveTerm(null, terms.value, device.lastTerm),
  );
  const term = resource(() => (termId === null ? null : catalog.term(termId)));
  const index = resource(() => catalog.catalogIndex());
  const offerings = resource(() => catalog.offerings());
  const fce = resource(() => catalog.fceSummary());
  const current = currentTerm();
  const lookup = $derived(courseLookup(term.value ?? null, index.value ?? null));
  // Pickers run through the newest built semester, or today's when that's later.
  const latest = $derived.by(() => {
    const newest = terms.value?.[0]?.id;
    if (newest === undefined) return current;
    return (termOrdinal(newest) ?? 0) >= (termOrdinal(current) ?? 0) ? newest : current;
  });

  return {
    get terms() {
      return terms.value ?? [];
    },
    get term() {
      return term.value ?? null;
    },
    get index() {
      return index.value ?? null;
    },
    get offerings() {
      return offerings.value ?? null;
    },
    get fce() {
      return fce.status === "ready" ? (fce.value ?? null) : null;
    },
    current,
    lookup: (id) => lookup(id),
    facts: (id) => {
      const known = term.value?.courseById.get(id) ?? offerings.value?.byId.get(id);
      if (known === undefined) return {};
      return { dept: known.dept, level: known.level === "G" ? "G" : "U" };
    },
    hours: (id) => (fce.status === "ready" ? (fce.value?.courses.get(id)?.hours ?? null) : null),
    termChoices: (entryTerm) => termOptions(latest, entryTerm),
  };
}
