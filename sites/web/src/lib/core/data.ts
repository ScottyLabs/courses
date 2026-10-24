/* oxlint-disable eslint/max-depth, eslint/max-lines-per-function, eslint/require-unicode-regexp, unicorn/no-array-sort, unicorn/prefer-at, unicorn/prefer-string-replace-all -- Courseweave's code, ported unchanged (decision 14) so fixes carry over between the two codebases */
// A loaded term with lookup tables. The loaders live in lib/data (catalog documents from the API).
import { departmentName } from "./departments.ts";
import { buildSearchDocs, normalize, type SearchDoc } from "./search.ts";
import { teachesSection } from "./sections.ts";
import type { Course, TermData, TermMeta } from "./types.ts";

export interface Instructor {
  name: string;
  slug: string;
  courseIds: string[];
  /** Number of sections taught. */
  sectionCount: number;
  depts: string[];
}

export interface Department {
  code: string;
  name: string;
  /** Course-number prefixes used by this department, e.g. ["15"]. */
  prefixes: string[];
  courseCount: number;
}

/** A loaded term with lookup tables. Built once per term and shared app-wide. */
export interface Term {
  meta: TermMeta;
  generatedAt: string;
  courses: Course[];
  courseById: Map<string, Course>;
  searchDocs: SearchDoc[];
  instructors: Instructor[];
  instructorBySlug: Map<string, Instructor>;
  departments: Department[];
}

export function instructorSlug(name: string): string {
  return normalize(name)
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

export function indexTerm(data: TermData): Term {
  const courseById = new Map(data.courses.map((c) => [c.id, c]));

  const instructorMap = new Map<string, Instructor>();
  const usedSlugs = new Set<string>();
  for (const course of data.courses) {
    for (const section of course.sections) {
      for (const name of section.instructors) {
        // Recitations that only repeat their lecture's instructors aren't extra teaching.
        if (!teachesSection(course, section, name)) continue;
        let inst = instructorMap.get(name);
        if (!inst) {
          let slug = instructorSlug(name);
          for (let n = 2; usedSlugs.has(slug); n++) slug = `${instructorSlug(name)}-${n}`;
          usedSlugs.add(slug);
          inst = { name, slug, courseIds: [], sectionCount: 0, depts: [] };
          instructorMap.set(name, inst);
        }
        inst.sectionCount++;
        if (!inst.courseIds.includes(course.id)) inst.courseIds.push(course.id);
        if (!inst.depts.includes(course.dept)) inst.depts.push(course.dept);
      }
    }
  }
  const instructors = [...instructorMap.values()].sort(
    (a, b) => lastName(a.name).localeCompare(lastName(b.name)) || a.name.localeCompare(b.name),
  );

  const deptMap = new Map<string, Department>();
  for (const course of data.courses) {
    let d = deptMap.get(course.dept);
    if (!d) {
      d = { code: course.dept, name: departmentName(course.dept), prefixes: [], courseCount: 0 };
      deptMap.set(course.dept, d);
    }
    d.courseCount++;
    const prefix = course.id.slice(0, 2);
    if (!d.prefixes.includes(prefix)) d.prefixes.push(prefix);
  }
  const departments = [...deptMap.values()].sort((a, b) => a.name.localeCompare(b.name));

  return {
    meta: data.term,
    generatedAt: data.generatedAt,
    courses: data.courses,
    courseById,
    searchDocs: buildSearchDocs(data.courses),
    instructors,
    instructorBySlug: new Map(instructors.map((i) => [i.slug, i])),
    departments,
  };
}

export function lastName(name: string): string {
  const parts = name.trim().split(/\s+/);
  return parts[parts.length - 1] ?? name;
}

/** Accepts "15-122" or "15122". */
export function parseCourseCode(code: string): string {
  return code.replace(/\D/g, "");
}
