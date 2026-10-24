// The profile's "Courses Taken" and "Currently Taking" lists: which record entries they show, how the department,
// course level and semester filters narrow them, and how they group by semester. Pure; the components pass in what
// they know about each course.
import { departmentName } from "../departments.ts";
import { earnsCredit, normalizeGrade, termOrdinal } from "../requisites.ts";
import type { RecordEntry } from "../state/record.ts";

export type CourseLevel = "U" | "G";

/** What the lists need to know about a course beyond the record entry. */
export interface CourseFacts {
  /** Department code from the Schedule of Classes ("CS"), when a semester or the offerings list knows it. */
  dept?: string | null;
  /** Undergraduate or graduate, when known. */
  level?: CourseLevel | null;
}

export type FactsOf = (courseId: string) => CourseFacts;

export interface ListFilters {
  /** Department key from departmentOptions(), or "" for all. */
  department: string;
  /** "U", "G" or "" for all. */
  level: string;
  /** Term code, NO_TERM for entries without one, or "" for all. */
  semester: string;
}

export const ALL_FILTERS: ListFilters = { department: "", level: "", semester: "" };

/** The semester filter's value for entries with no semester. */
export const NO_TERM = "none";

/**
 * A course's level: the Schedule of Classes' when known, else from its number (CMU's graduate courses are numbered
 * 600 and up within their department, as in 15-619).
 */
export function courseLevel(courseId: string, facts: CourseFacts = {}): CourseLevel {
  if (facts.level === "U" || facts.level === "G") return facts.level;
  return Number(courseId.slice(2, 3)) >= 6 ? "G" : "U";
}

/**
 * The department a course is filtered under: its Schedule of Classes code when known ("CS"), else its number prefix
 * ("15"), so courses no semester lists still group with their department's other numbers.
 */
export function departmentKey(courseId: string, facts: CourseFacts = {}): string {
  return facts.dept ?? courseId.slice(0, 2);
}

/** A department key's name: "Computer Science" for "CS", "Courses numbered 15-xxx" for a bare prefix. */
export function departmentLabel(key: string): string {
  return /^\d{2}$/u.test(key) ? `Courses numbered ${key}-xxx` : departmentName(key);
}

/** Courses taken at CMU, newest semester first, then by course number. Entries without a semester come last. */
export function takenEntries(entries: RecordEntry[]): RecordEntry[] {
  return entries
    .filter((e) => e.source === "taken")
    .toSorted(
      (a, b) =>
        (termOrdinal(b.term ?? "") ?? 0) - (termOrdinal(a.term ?? "") ?? 0) ||
        (a.courseId ?? "").localeCompare(b.courseId ?? ""),
    );
}

/** Whether an entry passes the filters. Entries without a course (none are "taken") never match a department. */
export function matchesFilters(
  entry: RecordEntry,
  filters: ListFilters,
  factsOf: FactsOf,
): boolean {
  const id = entry.courseId;
  if (filters.semester !== "") {
    const term = entry.term ?? NO_TERM;
    if (term !== filters.semester) return false;
  }
  if (id === null) return filters.department === "" && filters.level === "";
  const facts = factsOf(id);
  if (filters.department !== "" && departmentKey(id, facts) !== filters.department) return false;
  if (filters.level !== "" && courseLevel(id, facts) !== filters.level) return false;
  return true;
}

export interface Choice {
  value: string;
  label: string;
}

/** The departments among these entries, by name. */
export function departmentOptions(entries: RecordEntry[], factsOf: FactsOf): Choice[] {
  const keys = new Set<string>();
  for (const e of entries)
    if (e.courseId !== null) keys.add(departmentKey(e.courseId, factsOf(e.courseId)));
  return [...keys]
    .map((value) => ({ value, label: departmentLabel(value) }))
    .toSorted((a, b) => a.label.localeCompare(b.label));
}

/** The levels among these entries, undergraduate first. */
export function levelOptions(entries: RecordEntry[], factsOf: FactsOf): Choice[] {
  const levels = new Set<CourseLevel>();
  for (const e of entries)
    if (e.courseId !== null) levels.add(courseLevel(e.courseId, factsOf(e.courseId)));
  const out: Choice[] = [];
  if (levels.has("U")) out.push({ value: "U", label: "Undergraduate" });
  if (levels.has("G")) out.push({ value: "G", label: "Graduate" });
  return out;
}

/** The semesters among these entries, newest first; "Semester not set" last when some entries have none. */
export function semesterOptions(entries: RecordEntry[], label: (term: string) => string): Choice[] {
  const terms = new Set<string>();
  let unset = false;
  for (const e of entries) {
    if (e.term === undefined || e.term === "") unset = true;
    else terms.add(e.term);
  }
  const out = [...terms]
    .toSorted((a, b) => (termOrdinal(b) ?? 0) - (termOrdinal(a) ?? 0))
    .map((value) => ({ value, label: label(value) }));
  if (unset) out.push({ value: NO_TERM, label: "Semester not set" });
  return out;
}

export interface TermGroup {
  /** Term code, or "" for entries without one. */
  term: string;
  entries: RecordEntry[];
}

/** Entries grouped by semester, keeping their order (takenEntries() sorts them newest first). */
export function groupByTerm(entries: RecordEntry[]): TermGroup[] {
  const groups = new Map<string, RecordEntry[]>();
  for (const e of entries) {
    const key = e.term ?? "";
    groups.set(key, [...(groups.get(key) ?? []), e]);
  }
  return [...groups].map(([term, list]) => ({ term, entries: list }));
}

/**
 * Courses the student is taking in `term`: taken entries for that semester that haven't ended in a grade that
 * earns no credit (a withdrawal, say). Usually they have no grade yet.
 */
export function currentlyTaking(entries: RecordEntry[], term: string): RecordEntry[] {
  return takenEntries(entries).filter((e) => {
    if (e.term !== term || e.courseId === null) return false;
    const grade = normalizeGrade(e.grade);
    return grade === undefined || earnsCredit(grade);
  });
}

/** Course ids in `scheduled` that the record doesn't have for `term`, in schedule order and without repeats. */
export function missingFromRecord(
  entries: RecordEntry[],
  term: string,
  scheduled: string[],
): string[] {
  const have = new Set(
    entries
      .filter((e) => e.source === "taken" && e.term === term && e.courseId !== null)
      .map((e) => e.courseId),
  );
  return [...new Set(scheduled)].filter((id) => !have.has(id));
}
