// Links for the course feature that core/routes.ts doesn't have yet.
import { codeOf } from "$lib/core/requisite-format";

/** The requisite explorer's views: the list (tree and unlocks) and the graph. */
export type RequisitesView = "list" | "graph";

/** The requisite explorer, centered on a course (five-digit id or code) when one is given, in `view` when given. */
export function requisitesPath(
  course?: string | null,
  term?: string | null,
  view?: RequisitesView | null,
): string {
  const code =
    course === null || course === undefined
      ? ""
      : /^\d{5}$/u.test(course)
        ? codeOf(course)
        : course;
  const path = code === "" ? "/requisites" : `/requisites/${encodeURIComponent(code)}`;
  const params = new URLSearchParams();
  if (term !== null && term !== undefined && term !== "") params.set("term", term);
  if (view !== null && view !== undefined) params.set("view", view);
  const query = params.toString();
  return query === "" ? path : `${path}?${query}`;
}
