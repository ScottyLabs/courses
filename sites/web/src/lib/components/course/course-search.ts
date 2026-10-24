// Searching any course, not only a semester's: the whole catalog as search docs (built once per catalog), merged with
// a semester's courses. The degree plan's "Add a course" ranks the catalog first and adds the semester's special
// listings the catalog doesn't know; the record's course picker and the requisite explorer rank the semester's
// courses first, then the rest of the catalog. Courseweave's AddCourseInput, CoursePicker and RequisiteSearch.
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

/** Up to `limit` matches from `first`, then from `then` without repeats. */
function merged(first: SearchDoc[], then: SearchDoc[], query: string, limit: number): Course[] {
  if (query.trim() === "") return [];
  const head = searchCourses(first, query, limit);
  if (head.length >= limit) return head;
  const seen = new Set(head.map((c) => c.id));
  const tail = searchCourses(then, query, limit).filter((c) => !seen.has(c.id));
  return [...head, ...tail].slice(0, limit);
}

/** Up to `limit` courses matching `query`: catalog courses first, then the semester's courses the catalog lacks. */
export function findCourses(
  catalog: CatalogIndex | null,
  termDocs: SearchDoc[],
  query: string,
  limit = 8,
): Course[] {
  return merged(catalog === null ? [] : catalogDocs(catalog), termDocs, query, limit);
}

/** Up to `limit` courses matching `query`: the semester's courses first, then the rest of the catalog. */
export function findOfferedFirst(
  termDocs: SearchDoc[],
  catalog: CatalogIndex | null,
  query: string,
  limit = 8,
): Course[] {
  return merged(termDocs, catalog === null ? [] : catalogDocs(catalog), query, limit);
}
