/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../testing.ts";
import type { TermIndexEntry } from "../core/types.ts";
import { semesterOptions } from "./term-scope.ts";

describe("semesterOptions", () => {
  it("lists the built semesters in order, then All semesters", () => {
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- only id and name matter here
    const terms = [
      { id: "S27", name: "Spring 2027" },
      { id: "F26", name: "Fall 2026" },
    ] as TermIndexEntry[];
    expect(semesterOptions(terms)).toEqual([
      { value: "S27", label: "Spring 2027" },
      { value: "F26", label: "Fall 2026" },
      { value: "all", label: "All semesters" },
    ]);
    expect(semesterOptions([])).toEqual([{ value: "all", label: "All semesters" }]);
  });
});
