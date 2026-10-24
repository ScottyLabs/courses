// An instructor's FCE rows: every row of theirs from the courses the FCE index lists for them, loaded in the browser
// (decision 39), for instructor cards and pages. The window and the averages are the shared course ones
// (components/course/fce-aggregate.ts).
import { weightedMean, type FceRow, type InstructorFce } from "../../core/fce.ts";
import { sameInstructor, termOrder, type NameIndex } from "../../core/fce-names.ts";

/** One FCE row for one of the instructor's courses. */
export interface InstructorRow extends FceRow {
  courseId: string;
}

/** Overall teaching rating over the last `recent` semesters with responses, weighted by responses. */
export function recentTeachingRate(fce: InstructorFce, recent = 4): number | null {
  return weightedMean(
    fce.terms.slice(-recent),
    (t) => t.teachingRate,
    (t) => t.responses,
  );
}

/** What loading an instructor's FCE rows needs from the catalog (lib/data's `catalog`). */
export interface FceSource {
  fceInstructors(): Promise<NameIndex<InstructorFce> | null>;
  fceCourses(prefix: string): Promise<Map<string, FceRow[]>>;
}

export interface InstructorFceData {
  /** The FCE index's entry for them: per-course totals and teaching rating by term. */
  summary: InstructorFce;
  /** Every row for their courses, newest first. */
  rows: InstructorRow[];
}

/**
 * An instructor's FCE history, by schedule-style name ("David Kosbie"): their entry in the FCE index and every row of
 * theirs from the courses it lists. Null when the catalog has no FCEs or none match the name.
 */
export async function loadInstructorFce(
  source: FceSource,
  name: string,
): Promise<InstructorFceData | null> {
  const index = await source.fceInstructors();
  const summary = index?.get(name);
  if (summary === undefined) return null;
  const courseIds = summary.courses.map((course) => course.courseId);
  const prefixes = [...new Set(courseIds.map((id) => id.slice(0, 2)))];
  const chunks = await Promise.all(prefixes.map((prefix) => source.fceCourses(prefix)));
  const byCourse = new Map<string, FceRow[]>();
  for (const chunk of chunks) for (const [id, rows] of chunk) byCourse.set(id, rows);
  const rows = courseIds.flatMap((courseId) =>
    (byCourse.get(courseId) ?? [])
      .filter((row) => sameInstructor(row.instructor, name))
      .map((row): InstructorRow => Object.assign({ courseId }, row)),
  );
  return {
    summary,
    rows: rows.toSorted(
      (a, b) => termOrder(b.term) - termOrder(a.term) || a.courseId.localeCompare(b.courseId),
    ),
  };
}
