// Finding any catalog course to add to a plan: the whole catalog, plus the browsed semester's special listings
// the catalog doesn't know. Courseweave's features/degree/AddCourseInput.tsx search.
import type { CatalogIndex } from "../../core/catalog-types.ts";
import { buildSearchDocs, searchCourses, type SearchDoc } from "../../core/search.ts";
import type { Course } from "../../core/types.ts";

const docsCache = new WeakMap<CatalogIndex, SearchDoc[]>();

/** Search docs for every catalog course, built once per catalog. */
export function catalogDocs(index: CatalogIndex): SearchDoc[] {
  let docs = docsCache.get(index);
  if (docs === undefined) {
    const courses: Course[] = Object.values(index.courses).map((e) => ({
      id: e.id,
      code: e.code,
      title: e.title,
      units: e.units,
      dept: "",
      level: "U",
      instructors: [],
      sections: [],
      crossListed: [],
    }));
    docs = buildSearchDocs(courses);
    docsCache.set(index, docs);
  }
  return docs;
}

/** Up to `limit` courses matching `query`: catalog courses first, then the semester's courses the catalog lacks. */
export function findCourses(
  catalog: CatalogIndex | null,
  termDocs: SearchDoc[],
  query: string,
  limit = 8,
): Course[] {
  if (query.trim() === "") return [];
  const fromCatalog = catalog === null ? [] : searchCourses(catalogDocs(catalog), query, limit);
  if (fromCatalog.length >= limit) return fromCatalog;
  const seen = new Set(fromCatalog.map((c) => c.id));
  const extra = searchCourses(termDocs, query, limit).filter((c) => !seen.has(c.id));
  return [...fromCatalog, ...extra].slice(0, limit);
}
