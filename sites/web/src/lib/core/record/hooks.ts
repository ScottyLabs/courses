import type { CatalogIndex } from "../catalog-types.ts";
import type { Term } from "../data.ts";
import { codeOf } from "../requisite-format.ts";
import { termLabel, termOrdinal, termRange } from "../requisites.ts";

export interface CourseInfo {
  id: string;
  code: string;
  title: string | undefined;
  /** Catalog units (minimum for variable-unit courses), or null if unknown. */
  units: number | null;
  passFail: boolean;
}

/** Title/units for any course id, from the catalog first and then this term's schedule. */
export function courseLookup(
  term: Term | null,
  catalog: CatalogIndex | null,
): (id: string) => CourseInfo {
  const courses = catalog?.courses ?? null;
  return (id: string) => {
    const c = courses?.[id];
    const t = term?.courseById.get(id);
    return {
      id,
      code: codeOf(id),
      title: c?.title ?? t?.title,
      units: c?.units ?? t?.units ?? null,
      passFail: c?.passFail ?? t?.passFail ?? false,
    };
  };
}

/** Term codes offered in pickers: newest first, from a few years before `termId` through it. */
export function termOptions(termId: string, entryTerm?: string): string[] {
  const current = termOrdinal(termId) ?? 0;
  const year = Math.floor(current / 3);
  const entry = entryTerm === undefined ? null : termOrdinal(entryTerm);
  const from =
    entryTerm !== undefined && entry !== null && entry < current
      ? entryTerm
      : `F${String((year - 5) % 100).padStart(2, "0")}`;
  return termRange(from, termId, true).toReversed();
}

export { termLabel };
