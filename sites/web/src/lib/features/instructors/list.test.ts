/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../../testing.ts";
import { indexTerm } from "../../core/data.ts";
import type { InstructorFce } from "../../core/fce.ts";
import { NameIndex } from "../../core/fce-names.ts";
import type { Course, Section, TermData } from "../../core/types.ts";
import {
  decodeInstructorList,
  listFromTerm,
  type InstructorListFile,
  type ListedDoc,
} from "./list.ts";

// Made-up people and courses.
function section(name: string, instructors: string[], extra: Partial<Section> = {}): Section {
  return {
    name,
    kind: "lecture",
    parent: null,
    mini: null,
    mode: "IPE",
    instructors,
    meetings: [],
    ...extra,
  };
}

function course(id: string, title: string, sections: Section[]): Course {
  return {
    id,
    code: `${id.slice(0, 2)}-${id.slice(2)}`,
    title,
    units: 12,
    dept: "CS",
    level: "U",
    instructors: [],
    sections,
    crossListed: [],
  };
}

// Two people whose names make the same slug, and a topics course.
const data: TermData = {
  term: {
    id: "S27",
    name: "Spring 2027",
    season: "spring",
    year: 2027,
    minis: [],
    classesBegin: "2027-01-11",
    classesEnd: "2027-04-30",
    noClasses: [],
  },
  courses: [
    course("15122", "Imperative Computation", [
      section("1", ["Ada Example"]),
      section("A", ["Ada Example"], { kind: "section", parent: "1" }),
    ]),
    course("15390", "Special Topics", [
      section("A", ["Ada Ex\u00E4mple", "Ada Example"], { title: "Robotics" }),
    ]),
  ],
};

// What courses-index writes for that semester (crates/courses-index/src/instructors.rs).
const file: InstructorListFile = {
  titles: { "15122": "Imperative Computation", "15390": "Special Topics" },
  instructors: [
    {
      name: "Ada Example",
      depts: ["CS"],
      courses: ["15122", "15390"],
      sections: 2,
      topics: { "15390": ["Robotics"] },
      fce: { rating: 4.5, courses: ["15122"] },
    },
    // Accents aside, the same name: the same FCE record.
    {
      name: "Ada Ex\u00E4mple",
      depts: ["CS"],
      courses: ["15390"],
      sections: 1,
      topics: { "15390": ["Robotics"] },
      fce: { rating: 4.5, courses: ["15122"] },
    },
  ],
};

function rowKeys(rows: readonly ListedDoc[]): unknown[][] {
  return rows
    .map((d) => [
      d.inst.slug,
      d.inst.courseIds,
      d.inst.sectionCount,
      d.titles,
      d.inst.rating,
      d.inst.fceCourses,
    ])
    .toSorted((a, b) => String(a[0]).localeCompare(String(b[0])));
}

describe("decodeInstructorList", () => {
  it("numbers repeated slugs in the document's order", () => {
    const docs = decodeInstructorList(file);
    expect(docs.map((d) => d.inst.slug)).toEqual(["ada-example", "ada-example-2"]);
    expect(docs[0]?.inst.rating).toBe(4.5);
    expect(docs[0]?.inst.fceCourses).toEqual(["15122"]);
    expect(docs[0]?.inst.terms).toBeNull();
    expect(docs[0]?.letter).toBe("E");
  });

  it("searches course and topic titles", () => {
    expect(decodeInstructorList(file)[0]?.titles).toBe(
      "imperative computation | special topics | robotics",
    );
  });
});

describe("listFromTerm", () => {
  it("gives the same rows as the list document", () => {
    const fce = new NameIndex<InstructorFce>();
    fce.add("EXAMPLE, ADA", {
      name: "Ada Example",
      courses: [
        {
          courseId: "15122",
          terms: 1,
          responses: 10,
          teachingRate: 4.5,
          courseRate: 4,
          hours: 9,
          last: "F26",
          first: "F26",
        },
      ],
      terms: [{ term: "F26", teachingRate: 4.5, responses: 10 }],
    });
    expect(rowKeys(listFromTerm(indexTerm(data), fce))).toEqual(
      rowKeys(decodeInstructorList(file)),
    );
  });
});

describe("decodeInstructorList for all semesters", () => {
  it("gives all-semesters rows the semesters they teach in, and none an FCE record without a match", () => {
    const docs = decodeInstructorList({
      titles: {},
      instructors: [
        { name: "Ada Example", depts: [], courses: [], sections: 0, terms: ["S27", "F26"] },
      ],
    });
    expect(docs[0]?.inst.terms).toEqual(["S27", "F26"]);
    expect(docs[0]?.inst.fceCourses).toBeNull();
    expect(docs[0]?.inst.rating).toBeNull();
  });
});
