import { describe, it } from "node:test";
import { expect } from "../../testing.ts";
import { move, typeahead } from "./listbox.ts";

const terms = [
  { value: "S27", label: "Spring 2027" },
  { value: "F26", label: "Fall 2026" },
  { value: "M26", label: "Summer 2026", disabled: true },
  { value: "S26", label: "Spring 2026" },
  { value: "F25", label: "Fall 2025" },
  { value: "all", label: "All semesters" },
];

void describe("move", () => {
  void it("steps over disabled options and stops at the ends", () => {
    expect(move(terms, 1, 1)).toBe(3);
    expect(move(terms, 3, -1)).toBe(1);
    expect(move(terms, 5, 1)).toBe(5);
    expect(move(terms, 0, -1)).toBe(0);
  });
  void it("finds the first and last enabled options", () => {
    expect(move(terms, -1, 1)).toBe(0);
    expect(move(terms, terms.length, -1)).toBe(5);
    expect(move([{ value: "x", label: "X", disabled: true }], -1, 1)).toBe(-1);
  });
  void it("moves a page at a time", () => {
    expect(move(terms, 0, 1, 3)).toBe(4);
    expect(move(terms, 0, 1, 10)).toBe(5);
    expect(move(terms, 5, -1, 10)).toBe(0);
  });
});

void describe("typeahead", () => {
  void it("matches the start of a label, ignoring case", () => {
    expect(typeahead(terms, "f", -1)).toBe(1);
    expect(typeahead(terms, "AL", -1)).toBe(5);
    expect(typeahead(terms, "x", 0)).toBe(-1);
  });
  void it("goes to the next match when the same first letter is typed again", () => {
    expect(typeahead(terms, "s", 0)).toBe(3);
    expect(typeahead(terms, "s", 3)).toBe(0);
    expect(typeahead(terms, "sss", 0)).toBe(3);
  });
  void it("keeps the active option while a longer query still matches it", () => {
    expect(typeahead(terms, "fa", 1)).toBe(1);
    expect(typeahead(terms, "fall 2025", 1)).toBe(4);
  });
  void it("skips disabled options", () => {
    expect(typeahead(terms, "su", -1)).toBe(-1);
  });
});
