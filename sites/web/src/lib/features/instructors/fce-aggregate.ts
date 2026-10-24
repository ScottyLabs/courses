// An instructor's FCE rows and the "Aggregate FCEs" figures the Figma shows on instructor cards: workload, teaching
// rate and course rate over the most recent semesters the student chose, in the seasons they chose, optionally for
// some courses only. Means are weighted by responses, as Courseweave's FCE summaries are.
import { weightedMean, type FceRow, type InstructorFce } from "../../core/fce.ts";
import { sameInstructor, termOrder, type NameIndex } from "../../core/fce-names.ts";

export type Season = "spring" | "summer" | "fall";

export const SEASONS: readonly Season[] = ["spring", "summer", "fall"];

export interface AggregateSettings {
  /** How many of the most recent semesters to count; null counts every semester. */
  semesters: number | null;
  /** Which seasons count. */
  seasons: Record<Season, boolean>;
}

export const DEFAULT_AGGREGATE: AggregateSettings = {
  semesters: 2,
  seasons: { spring: true, summer: false, fall: true },
};

/** Most semesters the "Semesters to show" field accepts. */
export const MAX_SEMESTERS = 40;

/** One FCE row for one of the instructor's courses. */
export interface InstructorRow extends FceRow {
  courseId: string;
}

export interface FceAggregate {
  /** Semesters with responses among the rows. */
  semesters: number;
  responses: number;
  /** Hours a week students reported. */
  hours: number | null;
  teachingRate: number | null;
  courseRate: number | null;
}

const SEASON_BY_LETTER: Record<string, Season> = { S: "spring", M: "summer", F: "fall" };

/** "S26" -> "spring"; null for anything else. */
export function seasonOf(term: string): Season | null {
  return SEASON_BY_LETTER[term.charAt(0)] ?? null;
}

/** Settings read back from storage, with defaults for anything missing or malformed. */
export function normalizeSettings(raw: unknown): AggregateSettings {
  if (typeof raw !== "object" || raw === null) return DEFAULT_AGGREGATE;
  const semesters: unknown = Reflect.get(raw, "semesters");
  const seasons: unknown = Reflect.get(raw, "seasons");
  const flag = (season: Season): boolean => {
    const value: unknown =
      typeof seasons === "object" && seasons !== null ? Reflect.get(seasons, season) : null;
    return typeof value === "boolean" ? value : DEFAULT_AGGREGATE.seasons[season];
  };
  return {
    semesters:
      semesters === null
        ? null
        : typeof semesters === "number" && Number.isInteger(semesters) && semesters >= 1
          ? Math.min(semesters, MAX_SEMESTERS)
          : DEFAULT_AGGREGATE.semesters,
    seasons: { spring: flag("spring"), summer: flag("summer"), fall: flag("fall") },
  };
}

function newestFirst<T extends { term: string }>(a: T, b: T): number {
  return termOrder(b.term) - termOrder(a.term);
}

/**
 * The rows the aggregate counts, newest first: rows with responses, in the chosen seasons and courses (all when
 * `courses` is empty), from the most recent `settings.semesters` semesters among them.
 */
export function windowRows<T extends { term: string; courseId: string; responses: number }>(
  rows: readonly T[],
  settings: AggregateSettings,
  courses: readonly string[] = [],
): T[] {
  const chosen = new Set(courses);
  const matching = rows.filter((row) => {
    const season = seasonOf(row.term);
    return (
      row.responses > 0 &&
      season !== null &&
      settings.seasons[season] &&
      (chosen.size === 0 || chosen.has(row.courseId))
    );
  });
  const terms = [...new Set(matching.map((row) => row.term))].toSorted(
    (a, b) => termOrder(b) - termOrder(a),
  );
  const kept = new Set(settings.semesters === null ? terms : terms.slice(0, settings.semesters));
  return matching.filter((row) => kept.has(row.term)).toSorted(newestFirst);
}

/** Response-weighted workload and ratings over `rows`, skipping rows without responses. */
const byResponses = (row: FceRow): number => row.responses;

export function aggregate(rows: readonly FceRow[]): FceAggregate {
  const answered = rows.filter((row) => row.responses > 0);
  return {
    semesters: new Set(answered.map((row) => row.term)).size,
    responses: answered.reduce((sum, row) => sum + row.responses, 0),
    hours: weightedMean(answered, (row) => row.hours, byResponses),
    teachingRate: weightedMean(answered, (row) => row.teachingRate, byResponses),
    courseRate: weightedMean(answered, (row) => row.courseRate, byResponses),
  };
}

/** Percent of enrolled students who responded, or null without an enrollment. */
export function responseRate(row: Pick<FceRow, "responses" | "enrolled">): number | null {
  return row.enrolled > 0 ? (row.responses / row.enrolled) * 100 : null;
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
    rows: rows.toSorted((a, b) => newestFirst(a, b) || a.courseId.localeCompare(b.courseId)),
  };
}
