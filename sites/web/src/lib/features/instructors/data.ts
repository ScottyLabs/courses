// Loaders shared by the instructor list and the instructor page, over lib/data's catalog.
import { catalog } from "$lib/data";
import {
  allInstructors,
  courseLookup,
  type AllInstructors,
  type CourseLookup,
} from "$lib/core/instructors/allInstructors";
import { termLabel } from "$lib/core/requisites";
import { decodeInstructorList, listFromAll, listFromTerm, type ListedDoc } from "./list";

/** Course titles and departments for any course, from the offerings index and the course catalog. */
export async function loadCourseLookup(): Promise<CourseLookup> {
  const [offerings, index] = await Promise.all([catalog.offerings(), catalog.catalogIndex()]);
  return courseLookup(offerings, index);
}

/**
 * Everyone teaching in any built semester plus everyone in the FCE history (Courseweave's "All semesters" list);
 * null when the catalog has no teaching document. FCEs are optional: without them the list is the teaching index.
 */
export async function loadAllInstructors(): Promise<AllInstructors | null> {
  const [teaching, lookup] = await Promise.all([catalog.teaching(), loadCourseLookup()]);
  const fce = await catalog.fceInstructors().catch(() => null);
  return allInstructors(teaching, fce, lookup);
}

/** "Spring 2027 and 3 more", "Course evaluations only": where an all-semesters instructor teaches, newest first. */
export function teachingSummary(terms: readonly string[]): string {
  const [latest] = terms;
  if (latest === undefined) return "Course evaluations only";
  const more = terms.length - 1;
  return more > 0 ? `${termLabel(latest)} and ${more} more` : termLabel(latest);
}

/**
 * The list's rows for a semester, or for all semesters with `termId` null: the compact list document (decision 43),
 * or the full documents when this catalog was built without it. Null for all semesters when the catalog has no
 * teaching document.
 */
export async function loadInstructorList(termId: string | null): Promise<ListedDoc[] | null> {
  const file = await catalog.instructorList(termId ?? "all");
  if (file !== null) return decodeInstructorList(file);
  const fce = catalog.fceInstructors().catch(() => null);
  if (termId !== null) return listFromTerm(await catalog.term(termId), await fce);
  const all = await loadAllInstructors();
  return all === null ? null : listFromAll(all, await fce);
}
