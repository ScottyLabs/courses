// The instructor list's rows: from the compact `instructors/<termId>` and `instructors/all` documents (decision 43),
// or, for a catalog built before them, from the full documents the list used to read. Either way each row has what
// the list shows and searches, and the same URL slug the instructor's page looks up.
import { instructorSlug, lastName, type Instructor, type Term } from "../../core/data.ts";
import type { InstructorFce } from "../../core/fce.ts";
import type { NameIndex } from "../../core/fce-names.ts";
import type { AllInstructors } from "../../core/instructors/allInstructors.ts";
import { buildInstructorDocs, type InstructorDoc } from "../../core/instructors/instructorData.ts";
import { normalize } from "../../core/search.ts";
import { recentTeachingRate } from "../../components/instructor/instructor-fce.ts";
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

function fceFields(
  name: string,
  fce: NameIndex<InstructorFce> | null,
): Pick<ListedInstructor, "rating" | "fceCourses"> {
  const entry = fce?.get(name);
  if (entry === undefined) return { rating: null, fceCourses: null };
  return {
    rating: recentTeachingRate(entry),
    fceCourses: entry.courses.map((course) => course.courseId).toSorted(),
  };
}

/** A row built from the full documents, as the list documents would give it. */
function listed(
  doc: InstructorDoc,
  terms: readonly string[] | null,
  fce: NameIndex<InstructorFce> | null,
): ListedDoc {
  const { name, slug, courseIds, sectionCount, depts } = doc.inst;
  return {
    inst: { name, slug, courseIds, sectionCount, depts, terms, ...fceFields(name, fce) },
    name: doc.name,
    titles: doc.titles,
    letter: doc.letter,
  };
}

/** The list's rows from a whole semester document and the FCE index, for catalogs without the list documents. */
export function listFromTerm(term: Term, fce: NameIndex<InstructorFce> | null): ListedDoc[] {
  return buildInstructorDocs(term).map((doc) => listed(doc, null, fce));
}

/** The list's rows from the all-semesters list, for catalogs without the list documents. */
export function listFromAll(
  all: AllInstructors,
  fce: NameIndex<InstructorFce> | null,
): ListedDoc[] {
  return all.docs.map((doc) =>
    listed(
      doc,
      (all.bySlug.get(doc.inst.slug)?.terms ?? []).map((t) => t.term),
      fce,
    ),
  );
}
