// Links for the course feature that core/routes.ts doesn't have yet.
import { codeOf } from "$lib/core/requisite-format";

/** The requisite explorer, centered on a course (five-digit id or code) when one is given. */
export function requisitesPath(course?: string | null, term?: string | null): string {
  const code =
    course === null || course === undefined
      ? ""
      : /^\d{5}$/u.test(course)
        ? codeOf(course)
        : course;
  const path = code === "" ? "/requisites" : `/requisites/${encodeURIComponent(code)}`;
  return term === null || term === undefined || term === ""
    ? path
    : `${path}?term=${encodeURIComponent(term)}`;
}
