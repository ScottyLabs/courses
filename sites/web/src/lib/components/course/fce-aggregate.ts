// The course card's "Aggregate Data": FCE rows narrowed to the semesters the student asked for (the rail's
// "Aggregate FCEs" options), then averaged by responses.
import { termOrder, weightedMean, type FceRow } from "$lib/core/fce";

export type FceSeasonLetter = "S" | "M" | "F";

export interface FceAggregateOptions {
  /** Most recent semesters with responses to include; null for all of them. */
  semesters: number | null;
  /** Seasons to include. */
  seasons: readonly FceSeasonLetter[];
}

export const DEFAULT_FCE_OPTIONS: FceAggregateOptions = { semesters: 2, seasons: ["S", "F"] };

export interface FceAggregate {
  hours: number | null;
  teachingRate: number | null;
  courseRate: number | null;
  responses: number;
  /** Semesters the averages cover, newest first. */
  terms: string[];
}

/** Rows from the chosen seasons and the most recent `semesters` of them that have responses, newest first. */
export function selectFceRows(rows: readonly FceRow[], options: FceAggregateOptions): FceRow[] {
  const seasons = new Set<string>(options.seasons);
  const inSeason = rows.filter((row) => seasons.has(row.term[0] ?? ""));
  const terms = [
    ...new Set(inSeason.filter((row) => row.responses > 0).map((row) => row.term)),
  ].toSorted((a, b) => termOrder(b) - termOrder(a));
  const kept = new Set(options.semesters === null ? terms : terms.slice(0, options.semesters));
  return inSeason
    .filter((row) => kept.has(row.term))
    .toSorted(
      (a, b) => termOrder(b.term) - termOrder(a.term) || a.section.localeCompare(b.section),
    );
}

const byResponses = (row: FceRow): number => row.responses;

/** Response-weighted averages over the rows. */
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

/** Parses stored options, falling back to the defaults for anything missing or malformed. */
export function parseFceOptions(raw: string | null): FceAggregateOptions {
  if (raw === null) return DEFAULT_FCE_OPTIONS;
  try {
    const value: unknown = JSON.parse(raw);
    if (typeof value !== "object" || value === null) return DEFAULT_FCE_OPTIONS;
    const semesters =
      "semesters" in value &&
      typeof value.semesters === "number" &&
      Number.isInteger(value.semesters) &&
      value.semesters > 0
        ? value.semesters
        : null;
    const seasons =
      "seasons" in value && Array.isArray(value.seasons)
        ? value.seasons.filter((s): s is FceSeasonLetter => s === "S" || s === "M" || s === "F")
        : DEFAULT_FCE_OPTIONS.seasons;
    return { semesters, seasons };
  } catch {
    return DEFAULT_FCE_OPTIONS;
  }
}
