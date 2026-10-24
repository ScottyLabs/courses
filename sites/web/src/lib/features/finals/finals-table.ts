// The registrar's final exam table: rows with the catalog's titles, search, department filter and sorting.
import type { RegistrarFinal } from "../../data/documents.ts";
import { examMinutes, fromRegistrar, type Exam } from "./finals.ts";

/** A row of the registrar's table: the exam with the catalog's title and department for search and filters. */
export interface FinalRow {
  exam: Exam;
  /** The course's full title when the semester has it, else the registrar's abbreviation. */
  courseTitle: string;
  /** Department code, such as "CS". */
  dept: string | null;
  instructors: string[];
}

export function finalRows(
  finals: readonly RegistrarFinal[],
  courseOf: (courseId: string) => { title: string; dept: string } | undefined,
): FinalRow[] {
  return finals.map((f) => {
    const course = courseOf(f.courseId);
    return {
      exam: fromRegistrar(f),
      courseTitle: course?.title ?? f.title,
      dept: course?.dept ?? null,
      instructors: f.instructors,
    };
  });
}

/** Rows matching every word of `query` (course number, title, instructor or room) and one of `depts` when given. */
export function filterFinals(
  rows: readonly FinalRow[],
  query: string,
  depts: readonly string[] = [],
): FinalRow[] {
  const words = query.toLowerCase().split(/\s+/u).filter(Boolean);
  return rows.filter((r) => {
    if (depts.length > 0 && (r.dept === null || !depts.includes(r.dept))) return false;
    const text = [
      r.exam.courseId,
      r.exam.code,
      `${r.exam.courseId}${r.exam.section ?? ""}`,
      r.exam.title,
      r.courseTitle,
      r.exam.location ?? "",
      ...r.instructors,
    ]
      .join(" ")
      .toLowerCase();
    return words.every(
      (w) => text.includes(w) || (/^\d\d-?\d+$/u.test(w) && text.includes(w.replace("-", ""))),
    );
  });
}

/** By course and section, or by when the exam is. */
export function sortFinals(rows: readonly FinalRow[], by: "course" | "date"): FinalRow[] {
  return rows.toSorted((a, b) =>
    by === "date"
      ? examMinutes(a.exam) - examMinutes(b.exam) || a.exam.courseId.localeCompare(b.exam.courseId)
      : a.exam.courseId.localeCompare(b.exam.courseId) ||
        (a.exam.section ?? "").localeCompare(b.exam.section ?? ""),
  );
}
