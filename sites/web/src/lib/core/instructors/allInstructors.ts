/* oxlint-disable eslint/max-lines-per-function, eslint/require-unicode-regexp, oxc/no-map-spread, typescript/strict-boolean-expressions, unicorn/no-array-sort -- Courseweave's code, ported unchanged (decision 14) so fixes carry over between the two codebases */
// The instructor explorer's "All semesters" view (`/instructors?term=all`): everyone teaching in any built semester,
// from the teaching index, plus (when FCEs are available) everyone in the FCE history who doesn't teach in one of them.
import type { CatalogIndex } from "../catalog-types.ts";
import { instructorSlug, lastName, type Instructor } from "../data.ts";
import type { InstructorFce } from "../fce.ts";
import { displayFceName, type NameIndex } from "../fce-names.ts";
import type { Offerings } from "../offerings.ts";
import { normalize } from "../search.ts";
import type { TeachingData } from "../types.ts";
import { catalogTitle } from "../explorer/unoffered.ts";
import type { InstructorDoc } from "./instructorData.ts";

export interface AllInstructor extends Instructor {
  /** Built semesters they teach in, newest first. Empty for people only in the FCE history. */
  terms: { term: string; courses: string[]; sections: number }[];
}

export interface CourseLookup {
  title(id: string): string | undefined;
  dept(id: string): string | undefined;
}

export interface AllInstructors {
  list: AllInstructor[];
  bySlug: Map<string, AllInstructor>;
  docs: InstructorDoc[];
}

/**
 * Everyone in `teaching`, then everyone in `fce` it doesn't match (FCE names, shown "First Last"). People only in the
 * FCE history get the courses it lists, and departments from those courses.
 */
export function buildAllInstructors(
  teaching: TeachingData,
  fce: NameIndex<InstructorFce> | null,
  course: CourseLookup,
): AllInstructors {
  const list: AllInstructor[] = [];
  const used = new Set<string>();
  const slugFor = (name: string) => {
    let slug = instructorSlug(name);
    for (let n = 2; used.has(slug); n++) slug = `${instructorSlug(name)}-${n}`;
    used.add(slug);
    return slug;
  };

  const matched = new Set<InstructorFce>();
  for (const [name, t] of Object.entries(teaching.instructors)) {
    const terms = teaching.terms
      .filter((id) => t.terms[id])
      .map((id) => ({ term: id, ...t.terms[id] }));
    const hit = fce?.get(name);
    if (hit) matched.add(hit);
    list.push({
      name,
      slug: slugFor(name),
      depts: t.depts,
      courseIds: [...new Set(terms.flatMap((x) => x.courses))].sort(),
      sectionCount: terms.reduce((n, x) => n + x.sections, 0),
      terms,
    });
  }
  for (const f of fce?.values() ?? []) {
    if (matched.has(f)) continue;
    const name = displayFceName(f.name);
    const courseIds = f.courses.map((c) => c.courseId).sort();
    const depts = [
      ...new Set(courseIds.map((id) => course.dept(id)).filter((d): d is string => Boolean(d))),
    ];
    list.push({ name, slug: slugFor(name), depts, courseIds, sectionCount: 0, terms: [] });
  }

  list.sort(
    (a, b) => lastName(a.name).localeCompare(lastName(b.name)) || a.name.localeCompare(b.name),
  );
  const docs = list.map((inst): InstructorDoc => {
    const first = normalize(lastName(inst.name)).charAt(0).toUpperCase();
    return {
      inst,
      name: normalize(inst.name),
      titles: normalize(inst.courseIds.map((id) => course.title(id) ?? "").join(" | ")),
      letter: /[A-Z]/.test(first) ? first : "#",
    };
  });
  return { list, bySlug: new Map(list.map((i) => [i.slug, i])), docs };
}

/** Course titles and departments from the offerings index and the catalog, for any course. */
export function courseLookup(
  offerings: Offerings | null,
  catalog: CatalogIndex | null,
): CourseLookup {
  const deptByPrefix = new Map<string, string>();
  for (const o of offerings?.byId.values() ?? [])
    if (!deptByPrefix.has(o.id.slice(0, 2))) deptByPrefix.set(o.id.slice(0, 2), o.dept);
  return {
    title: (id: string) =>
      offerings?.byId.get(id)?.title ?? catalogTitle(catalog?.courses[id]?.title),
    dept: (id: string) => offerings?.byId.get(id)?.dept ?? deptByPrefix.get(id.slice(0, 2)),
  };
}

/**
 * The all-semesters instructor list; null without a teaching document. Pass `fce` only when FCE access is
 * available: without it the list is the teaching index alone.
 */
export function allInstructors(
  teaching: TeachingData | null,
  fce: NameIndex<InstructorFce> | null,
  lookup: CourseLookup,
): AllInstructors | null {
  if (teaching === null) return null;
  return buildAllInstructors(teaching, fce, lookup);
}
