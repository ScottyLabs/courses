// The Figma's "Aggregate FCEs": FCE rows narrowed to the most recent semesters the student asked for (the rail's
// "Aggregate FCEs" options, shared by course and instructor cards), then averaged by responses, as Courseweave's FCE
// summaries are.
import { termOrder, weightedMean, type FceRow } from "../../core/fce.ts";

export type FceSeasonLetter = "S" | "M" | "F";

export interface FceAggregateOptions {
  /** Most recent semesters with responses to include; null for all of them. */
  semesters: number | null;
  /** Seasons to include. */
  seasons: readonly FceSeasonLetter[];
}

export const DEFAULT_FCE_OPTIONS: FceAggregateOptions = { semesters: 2, seasons: ["S", "F"] };

/** Most semesters the option accepts. */
export const MAX_FCE_SEMESTERS = 40;

export interface FceAggregate {
  hours: number | null;
  teachingRate: number | null;
  courseRate: number | null;
  responses: number;
  /** Semesters the averages cover, newest first. */
  terms: string[];
}

type WindowRow = Pick<FceRow, "term" | "responses" | "section"> & { courseId?: string };

/**
 * The rows the aggregate counts, newest first: rows with responses, in the chosen seasons and courses (all when
 * `courses` is empty), from the most recent `options.semesters` semesters among them.
 */
export function selectFceRows<T extends WindowRow>(
  rows: readonly T[],
  options: FceAggregateOptions,
  courses: readonly string[] = [],
): T[] {
  const seasons = new Set<string>(options.seasons);
  const chosen = new Set(courses);
  const matching = rows.filter(
    (row) =>
      row.responses > 0 &&
      seasons.has(row.term.charAt(0)) &&
      (chosen.size === 0 || (row.courseId !== undefined && chosen.has(row.courseId))),
  );
  const terms = [...new Set(matching.map((row) => row.term))].toSorted(
    (a, b) => termOrder(b) - termOrder(a),
  );
  const kept = new Set(options.semesters === null ? terms : terms.slice(0, options.semesters));
  return matching
    .filter((row) => kept.has(row.term))
    .toSorted(
      (a, b) =>
        termOrder(b.term) - termOrder(a.term) ||
        (a.courseId ?? "").localeCompare(b.courseId ?? "") ||
        a.section.localeCompare(b.section),
    );
}

const byResponses = (row: FceRow): number => row.responses;

/** Response-weighted averages over the rows, skipping rows without responses. */
export function aggregateFce(rows: readonly FceRow[]): FceAggregate {
  const answered = rows.filter((row) => row.responses > 0);
  return {
    hours: weightedMean(answered, (row) => row.hours, byResponses),
    teachingRate: weightedMean(answered, (row) => row.teachingRate, byResponses),
    courseRate: weightedMean(answered, (row) => row.courseRate, byResponses),
    responses: answered.reduce((sum, row) => sum + row.responses, 0),
    terms: [...new Set(answered.map((row) => row.term))].toSorted(
      (a, b) => termOrder(b) - termOrder(a),
    ),
  };
}

/** Percent of enrolled students who responded, or null without an enrollment. */
export function responseRate(row: Pick<FceRow, "responses" | "enrolled">): number | null {
  return row.enrolled > 0 ? (row.responses / row.enrolled) * 100 : null;
}

const SEASON_FLAGS: Record<string, FceSeasonLetter> = { spring: "S", summer: "M", fall: "F" };

function isLetter(value: unknown): value is FceSeasonLetter {
  return value === "S" || value === "M" || value === "F";
}

/** Seasons as stored: letters, or an older `{ spring: true, ... }` object. */
function parseSeasons(value: unknown): readonly FceSeasonLetter[] {
  if (Array.isArray(value)) return value.filter((item) => isLetter(item));
  if (typeof value !== "object" || value === null) return DEFAULT_FCE_OPTIONS.seasons;
  return Object.entries(SEASON_FLAGS)
    .filter(([name, letter]) => {
      const flag: unknown = Reflect.get(value, name);
      return typeof flag === "boolean" ? flag : DEFAULT_FCE_OPTIONS.seasons.includes(letter);
    })
    .map(([, letter]) => letter);
}

/** Options from anything: a missing or malformed field takes its default, and the semester count is capped. */
export function normalizeFceOptions(value: unknown): FceAggregateOptions {
  if (typeof value !== "object" || value === null) return DEFAULT_FCE_OPTIONS;
  const semesters: unknown = Reflect.get(value, "semesters");
  return {
    semesters:
      semesters === null
        ? null
        : typeof semesters === "number" && Number.isInteger(semesters) && semesters >= 1
          ? Math.min(semesters, MAX_FCE_SEMESTERS)
          : DEFAULT_FCE_OPTIONS.semesters,
    seasons: parseSeasons(Reflect.get(value, "seasons")),
  };
}

/** Options stored as JSON, falling back to the defaults. */
export function parseFceOptions(raw: string | null): FceAggregateOptions {
  if (raw === null) return DEFAULT_FCE_OPTIONS;
  try {
    return normalizeFceOptions(JSON.parse(raw));
  } catch {
    return DEFAULT_FCE_OPTIONS;
  }
}
