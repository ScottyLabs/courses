import type { CatalogIndex } from "../catalog-types.ts";
import type { Term } from "../data.ts";
import { coursePath } from "../routes.ts";

/** How a requisite course stands for the student. Supplied by the record/requisite checker. */
export type RequisiteStatus = "met" | "planned" | "missing";

export type StatusFn = (courseId: string) => RequisiteStatus | undefined;
export type HrefFn = (courseId: string) => string;

export interface ChipContext {
  termId: string;
  termName: string;
  offered(id: string): boolean;
  title(id: string): string | undefined;
  href: HrefFn;
  status?: StatusFn;
}

/** Lookup context for course chips: offered-this-term, titles, default links. */
export function chipContext(
  term: Term,
  catalog: CatalogIndex | null,
  status?: StatusFn,
  href?: HrefFn,
): ChipContext {
  const termId = term.meta.id;
  const courses = catalog?.courses ?? null;
  const offered = (id: string) => term.courseById.has(id);
  return {
    termId,
    termName: term.meta.name,
    offered,
    title: (id: string) => term.courseById.get(id)?.title ?? courses?.[id]?.title,
    // Courses not offered this term open their catalog page, which has the requisites and offering history.
    href: href ?? ((id: string) => coursePath(id, offered(id) ? termId : null)),
    status,
  };
}
