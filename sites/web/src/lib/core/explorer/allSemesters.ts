/* oxlint-disable unicorn/no-array-sort -- Courseweave's code, ported unchanged (decision 14) so fixes carry over between the two codebases */
// The explorer's "All semesters" view (`?term=all`): every course any built semester offers or the catalog lists,
// with FCEs and the semesters that offer it, but no sections. The rest of the app still runs on a real semester (the
// last one visited); see lib/state/term-scope.ts.
import type { CatalogIndex } from "../catalog-types.ts";
import { departmentName } from "../departments.ts";
import type { Department } from "../data.ts";
import type { Offerings } from "../offerings.ts";
import type { ScoredDoc } from "../search.ts";
import type { Course } from "../types.ts";
import { buildUnoffered } from "./unoffered.ts";

export interface AllCourses {
  docs: (ScoredDoc & { course: Course })[];
  /** Built semesters offering each course, newest first; empty for catalog-only courses. */
  termsOf: Map<string, string[]>;
  departments: Department[];
}

/**
 * Every course as a section-less `Course`, so the explorer's search, filters, sorting and FCE columns work on it.
 * Units are the catalog's (null for variable or unknown units, shown as VAR like a semester's own data).
 */
export function buildAllCourses(docs: ReturnType<typeof buildUnoffered>): AllCourses {
  const termsOf = new Map<string, string[]>();
  const deptMap = new Map<string, Department>();
  const out = docs.map((doc) => {
    const c = doc.course;
    termsOf.set(c.id, c.terms);
    if (c.dept) {
      let d = deptMap.get(c.dept);
      if (!d)
        deptMap.set(
          c.dept,
          (d = { code: c.dept, name: departmentName(c.dept), prefixes: [], courseCount: 0 }),
        );
      d.courseCount++;
      if (!d.prefixes.includes(c.id.slice(0, 2))) d.prefixes.push(c.id.slice(0, 2));
    }
    const course: Course = {
      id: c.id,
      code: c.code,
      title: c.title,
      units: c.units,
      dept: c.dept,
      level: c.level,
      instructors: [],
      sections: [],
      crossListed: [],
    };
    return { ...doc, course };
  });
  out.sort((a, b) => a.course.id.localeCompare(b.course.id));
  return {
    docs: out,
    termsOf,
    departments: [...deptMap.values()].sort((a, b) => a.name.localeCompare(b.name)),
  };
}

/** Every course for the all-semesters view; null when there's no offerings document (nothing to list). */
export function allCourses(
  offerings: Offerings | null,
  catalog: CatalogIndex | null,
): AllCourses | null {
  if (offerings === null) return null;
  return buildAllCourses(buildUnoffered(() => false, offerings, catalog));
}
