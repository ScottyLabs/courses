// Searching and labelling degree programs for the program picker. Courseweave's ProgramPicker logic.
import type { ProgramInfo, ProgramKind } from "../../../core/plan-types.ts";
import { normalize } from "../../../core/search.ts";

export const KIND_LABEL: Record<ProgramKind, string> = {
  major: "Major",
  minor: "Minor",
  "additional-major": "Additional major",
  requirements: "Requirements",
  track: "Track",
};

export type KindFilter = "all" | "major" | "minor" | "additional-major" | "other";

export const KIND_FILTERS: { value: KindFilter; label: string }[] = [
  { value: "all", label: "All" },
  { value: "major", label: "Majors" },
  { value: "minor", label: "Minors" },
  { value: "additional-major", label: "Additional majors" },
  { value: "other", label: "Concentrations and other" },
];

const KIND_RANK: ProgramKind[] = ["major", "additional-major", "minor", "track", "requirements"];

/** The campus a program belongs to when it isn't Pittsburgh ("Doha"), so same-named programs can be told apart. */
export function programCampus(p: ProgramInfo): string | null {
  const campuses = new Set(p.versions.flatMap((v) => (v.campus === null ? [] : [v.campus])));
  if (campuses.size !== 1 || campuses.has("Pittsburgh")) return null;
  return [...campuses][0] ?? null;
}

function matchesKind(p: ProgramInfo, filter: KindFilter): boolean {
  if (filter === "all") return true;
  if (filter === "other") return p.kind === "requirements" || p.kind === "track";
  return p.kind === filter;
}

/**
 * Programs whose names contain every word of `query`, leaving out the chosen ones: names starting with the query
 * first, then Pittsburgh before other campuses, majors before minors, then by name. With no query, a kind filter
 * lists every program of that kind; "all" lists nothing.
 */
export function searchPrograms(
  programs: ProgramInfo[],
  query: string,
  filter: KindFilter,
  chosen: Set<number>,
): ProgramInfo[] {
  const q = normalize(query).trim();
  const tokens = q.split(/\s+/u).filter((t) => t !== "");
  if (tokens.length === 0 && filter === "all") return [];
  return programs
    .filter((p) => {
      if (chosen.has(p.catalogId) || !matchesKind(p, filter)) return false;
      const name = normalize(p.name);
      return tokens.every((t) => name.includes(t));
    })
    .toSorted(
      (a, b) =>
        Number(normalize(b.name).startsWith(q)) - Number(normalize(a.name).startsWith(q)) ||
        Number(programCampus(a) !== null) - Number(programCampus(b) !== null) ||
        KIND_RANK.indexOf(a.kind) - KIND_RANK.indexOf(b.kind) ||
        a.name.localeCompare(b.name),
    );
}

export function countKind(programs: ProgramInfo[], kind: ProgramKind): number {
  return programs.filter((p) => p.kind === kind).length;
}
