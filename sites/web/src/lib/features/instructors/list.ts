// The instructor list's rows: from the compact `instructors/<termId>` and `instructors/all` documents (decision 43),
// or, for a catalog built before them, from the full documents the list used to read. Either way each row has what
// the list shows and searches, and the same URL slug the instructor's page looks up.
import type { Term } from "../../core/data.ts";
import type { InstructorFce } from "../../core/fce.ts";
import type { NameIndex } from "../../core/fce-names.ts";
import type { AllInstructors } from "../../core/instructors/allInstructors.ts";
import { buildInstructorDocs, type InstructorDoc } from "../../core/instructors/instructorData.ts";
import type { ListedDoc, ListedInstructor } from "../../components/instructor/directory.ts";
import { recentTeachingRate } from "../../components/instructor/instructor-fce.ts";

// The compact documents' rows are decoded in components/instructor/directory.ts, which previews share.
export {
  decodeInstructorList,
  type InstructorListFile,
  type ListedDoc,
  type ListedInstructor,
} from "../../components/instructor/directory.ts";

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
