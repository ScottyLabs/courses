/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import type { CatalogIndex } from "../../core/catalog-types.ts";
import { buildSearchDocs } from "../../core/search.ts";
import type { Course } from "../../core/types.ts";
import { expect } from "../../testing.ts";
import { catalogDocs, findCourses } from "./course-search.ts";

const entry = (id: string, title: string, units: number | null = 9) => ({
  id,
  code: `${id.slice(0, 2)}-${id.slice(2)}`,
  title,
  units,
  prereqs: null,
  coreqs: [],
  antireqs: [],
  equivalents: [],
  passFail: false,
});

const catalog: CatalogIndex = {
  courses: {
    "15122": entry("15122", "Principles of Imperative Computation", 12),
    "15150": entry("15150", "Principles of Functional Programming", 12),
    "21127": entry("21127", "Concepts of Mathematics"),
  },
};

const special: Course = {
  id: "98317",
  code: "98-317",
  title: "Student Taught Courses: Hype for Types",
  units: 3,
  dept: "STU",
  level: "U",
  instructors: [],
  sections: [],
  crossListed: [],
};

describe("findCourses", () => {
  it("finds catalog courses by number and title", () => {
    expect(findCourses(catalog, [], "15-122").map((c) => c.id)).toEqual(["15122"]);
    expect(
      findCourses(catalog, [], "principles")
        .map((c) => c.id)
        .toSorted(),
    ).toEqual(["15122", "15150"]);
  });

  it("adds the semester's courses the catalog lacks, once", () => {
    const termDocs = buildSearchDocs([
      special,
      { ...special, id: "15122", code: "15-122", title: "Principles of Imperative Computation" },
    ]);
    expect(findCourses(catalog, termDocs, "hype").map((c) => c.id)).toEqual(["98317"]);
    expect(findCourses(catalog, termDocs, "15-122").map((c) => c.id)).toEqual(["15122"]);
  });

  it("finds nothing for an empty query, and works without a catalog", () => {
    expect(findCourses(catalog, [], "  ")).toEqual([]);
    expect(findCourses(null, buildSearchDocs([special]), "98-317").map((c) => c.id)).toEqual([
      "98317",
    ]);
  });

  it("builds the catalog's search docs once", () => {
    expect(catalogDocs(catalog)).toBe(catalogDocs(catalog));
  });
});
