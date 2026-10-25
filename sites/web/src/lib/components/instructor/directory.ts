// The compact instructor documents (`instructors/<termId>`, `instructors/all`; decision 43) as rows: what the
// instructor list shows and searches, and each row's URL slug, given out in the document's order as the website's
// term index and all-semesters list give them out. The instructor list and instructor previews both read them.
import { instructorSlug, lastName, type Instructor } from "../../core/data.ts";
import type { InstructorDoc } from "../../core/instructors/instructorData.ts";
import { termLabel } from "../../core/requisites.ts";
import { normalize } from "../../core/search.ts";
import type { InstructorListFile } from "../../data/documents.ts";

export type { InstructorListFile };

/** An instructor as the list shows them. */
export interface ListedInstructor extends Instructor {
  /** Built semesters they teach in, newest first; null in a semester's list. */
  terms: readonly string[] | null;
  /** Recent teaching rating from FCEs. */
  rating: number | null;
  /** Courses with FCE rows for them; null when no FCE record matches the name. */
  fceCourses: readonly string[] | null;
}

export interface ListedDoc extends InstructorDoc {
  inst: ListedInstructor;
}

/** The letter heading a name sorts under, "#" for anything but A to Z. */
function letterOf(name: string): string {
  const first = normalize(lastName(name)).charAt(0).toUpperCase();
  return /[A-Z]/u.test(first) ? first : "#";
}

/** A slug for each name, numbered after the first as the website's term index and all-semesters list number them. */
function slugger(): (name: string) => string {
  const used = new Set<string>();
  return (name) => {
    const base = instructorSlug(name);
    let slug = base;
    for (let n = 2; used.has(slug); n++) slug = `${base}-${n}`;
    used.add(slug);
    return slug;
  };
}

/** The list's rows from a compact list document. */
export function decodeInstructorList(file: InstructorListFile): ListedDoc[] {
  const slugFor = slugger();
  return file.instructors.map((entry): ListedDoc => {
    const titles = entry.courses.flatMap((id) => [
      file.titles[id] ?? "",
      ...(entry.topics?.[id] ?? []),
    ]);
    return {
      inst: {
        name: entry.name,
        slug: slugFor(entry.name),
        courseIds: entry.courses,
        sectionCount: entry.sections,
        depts: entry.depts,
        terms: entry.terms ?? null,
        rating: entry.fce?.rating ?? null,
        fceCourses: entry.fce?.courses ?? null,
      },
      name: normalize(entry.name),
      titles: normalize(titles.join(" | ")),
      letter: letterOf(entry.name),
    };
  });
}

/** "Spring 2027 and 3 more", "Course evaluations only": where an all-semesters instructor teaches, newest first. */
export function teachingSummary(terms: readonly string[]): string {
  const [latest] = terms;
  if (latest === undefined) return "Course evaluations only";
  const more = terms.length - 1;
  return more > 0 ? `${termLabel(latest)} and ${more} more` : termLabel(latest);
}

/** One compact document, looked up the ways previews need. */
export interface InstructorDirectory {
  /** By URL slug. */
  bySlug: ReadonlyMap<string, ListedInstructor>;
  /** By name as schedules give it; the first of two people with one name. */
  byName: ReadonlyMap<string, ListedInstructor>;
  /** Course titles by course id. */
  titles: Readonly<Record<string, string>>;
}

export function instructorDirectory(file: InstructorListFile): InstructorDirectory {
  const bySlug = new Map<string, ListedInstructor>();
  const byName = new Map<string, ListedInstructor>();
  for (const { inst } of decodeInstructorList(file)) {
    bySlug.set(inst.slug, inst);
    if (!byName.has(inst.name)) byName.set(inst.name, inst);
  }
  return { bySlug, byName, titles: file.titles };
}

/** The instructor a preview names: by name when it has one, else (or failing that) by slug. */
export function findInstructor(
  directory: InstructorDirectory,
  target: { slug: string; name: string | null },
): ListedInstructor | undefined {
  const named = target.name === null ? undefined : directory.byName.get(target.name);
  return named ?? directory.bySlug.get(target.slug);
}
