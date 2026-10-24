// Loaders shared by the instructor list and the instructor page, over lib/data's catalog.
import { catalog } from "$lib/data";
import {
  allInstructors,
  courseLookup,
  type AllInstructors,
  type CourseLookup,
} from "$lib/core/instructors/allInstructors";
import { termLabel } from "$lib/core/requisites";

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

/** "Spring 2027 and 3 more", "Course evaluations only": where an all-semesters instructor teaches. */
export function teachingSummary(terms: readonly { term: string }[]): string {
  const [latest] = terms;
  if (latest === undefined) return "Course evaluations only";
  const more = terms.length - 1;
  return more > 0 ? `${termLabel(latest.term)} and ${more} more` : termLabel(latest.term);
}
