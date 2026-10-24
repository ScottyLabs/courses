// The Courses page's state in its URL: Courseweave's filters (core/explorer/filters.ts, same keys) plus the
// Figma's filters Courseweave doesn't have (a units range, a course-number level and the seasons a course is
// offered in), the search, the sort, the semester and the view options.
import {
  EMPTY_FILTERS,
  parseFilters,
  type Filters,
  type SortKey,
  writeFilters,
} from "../../core/explorer/filters.ts";
import type { CatalogEntry } from "../../core/catalog-types.ts";

export type Season = "fall" | "spring" | "summer";

export const SEASONS: readonly Season[] = ["fall", "spring", "summer"];

const isSeason = (value: string): value is Season =>
  value === "fall" || value === "spring" || value === "summer";

/** The slider's ends; a range covering both means "any units". */
export const UNITS_MIN = 0;
export const UNITS_MAX = 24;

export interface ExtraFilters {
  unitsLow: number;
  unitsHigh: number;
  /** Hundreds digit of the course number ("1" for 15-122), or null for any. */
  number: string | null;
  /** Seasons the course is offered in (any of them). */
  seasons: Season[];
}

export const EMPTY_EXTRA: ExtraFilters = {
  unitsLow: UNITS_MIN,
  unitsHigh: UNITS_MAX,
  number: null,
  seasons: [],
};

export interface ExplorerQuery {
  /** The `term` parameter as given: a semester id, "all", or null for the remembered one. */
  term: string | null;
  q: string;
  /** The `sort` parameter as given; parseSort() picks the default. */
  sort: string | null;
  /** The FCE hours chart instead of cards or rows. */
  chart: boolean;
  /** "Show courses not offered" in this semester (a view option: Reset keeps it). */
  unoffered: boolean;
  /** The course open in the detail panel ("15-122"), or null. */
  course: string | null;
  filters: Filters;
  extra: ExtraFilters;
}

function units(params: URLSearchParams, key: string, fallback: number): number {
  const raw = params.get(key);
  if (raw === null || raw === "") return fallback;
  const value = Number(raw);
  return Number.isInteger(value) && value >= UNITS_MIN && value <= UNITS_MAX ? value : fallback;
}

export function parseExplorer(params: URLSearchParams): ExplorerQuery {
  const low = units(params, "ulo", UNITS_MIN);
  const high = units(params, "uhi", UNITS_MAX);
  const number = params.get("num");
  return {
    term: params.get("term"),
    q: params.get("q") ?? "",
    sort: params.get("sort"),
    chart: params.get("view") === "chart",
    unoffered: params.get("unoffered") === "1",
    course: /^\d{2}-?\d{3}$/u.test(params.get("course") ?? "") ? params.get("course") : null,
    filters: parseFilters(params),
    extra: {
      unitsLow: Math.min(low, high),
      unitsHigh: Math.max(low, high),
      number: number !== null && /^[0-9]$/u.test(number) ? number : null,
      seasons: (params.get("offered") ?? "").split(",").filter((s) => isSeason(s)),
    },
  };
}

/** The query as URL parameters, leaving out defaults so links stay short. */
export function writeExplorer(query: ExplorerQuery): URLSearchParams {
  const params = writeFilters(new URLSearchParams(), query.filters);
  const put = (key: string, value: string | null): void => {
    if (value === null || value === "") params.delete(key);
    else params.set(key, value);
  };
  put("term", query.term);
  put("q", query.q.trim() === "" ? null : query.q);
  put("sort", query.sort);
  put("view", query.chart ? "chart" : null);
  put("unoffered", query.unoffered ? "1" : null);
  put("course", query.course);
  put("ulo", query.extra.unitsLow === UNITS_MIN ? null : String(query.extra.unitsLow));
  put("uhi", query.extra.unitsHigh === UNITS_MAX ? null : String(query.extra.unitsHigh));
  put("num", query.extra.number);
  put("offered", query.extra.seasons.join(","));
  // A stable order, so equal queries compare equal as strings.
  params.sort();
  return params;
}

export function unitsActive(extra: ExtraFilters): boolean {
  return extra.unitsLow !== UNITS_MIN || extra.unitsHigh !== UNITS_MAX;
}

export function extraFilterCount(extra: ExtraFilters): number {
  return (
    (unitsActive(extra) ? 1 : 0) +
    (extra.number === null ? 0 : 1) +
    (extra.seasons.length > 0 ? 1 : 0)
  );
}

export function hasExtraFilters(extra: ExtraFilters): boolean {
  return extraFilterCount(extra) > 0;
}

/** What the extra filters read about a course. */
export interface ExtraSubject {
  id: string;
  /** null for variable units. */
  units: number | null;
  /** Its catalog entry, for variable units' range and the seasons it's usually offered in. */
  entry?: CatalogEntry;
  /** Built semesters offering it ("F26"), for the seasons. */
  terms?: readonly string[];
}

const SEASON_LETTER: Record<Season, string> = { fall: "F", spring: "S", summer: "M" };

export function extraMatches(subject: ExtraSubject, extra: ExtraFilters): boolean {
  if (extra.number !== null && subject.id[2] !== extra.number) return false;
  if (unitsActive(extra)) {
    let low = subject.units;
    let high = subject.units;
    if (low === null) {
      // Variable units: the catalog's range overlaps the chosen one.
      low = subject.entry?.units ?? null;
      high = subject.entry?.unitsMax ?? low;
    }
    if (low === null || high === null) return false;
    if (high < extra.unitsLow || low > extra.unitsHigh) return false;
  }
  if (extra.seasons.length > 0) {
    const usually = subject.entry?.offered ?? [];
    const terms = subject.terms ?? [];
    const offered = extra.seasons.some(
      (season) =>
        usually.includes(season) || terms.some((term) => term.startsWith(SEASON_LETTER[season])),
    );
    if (!offered) return false;
  }
  return true;
}

/** The query with every filter cleared; the search, semester, sort and view options stay. */
export function clearFilters(query: ExplorerQuery): ExplorerQuery {
  return { ...query, filters: { ...EMPTY_FILTERS }, extra: { ...EMPTY_EXTRA } };
}

export const SORT_LABELS: Record<SortKey, string> = {
  relevance: "Relevance",
  number: "Number",
  title: "Title",
  units: "Units",
  hours: "Fewest FCE hours",
  "hours-desc": "Most FCE hours",
  ratio: "FCE hours vs units",
  rating: "Course rating",
  teaching: "Teaching rating",
};
