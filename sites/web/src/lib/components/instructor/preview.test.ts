/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../../testing.ts";
import { linkTarget } from "../course/preview.ts";
import { instructorDirectory, findInstructor, type InstructorListFile } from "./directory.ts";
import {
  instructorLinkTarget,
  instructorPreview,
  markedInstructor,
  pageInstructor,
} from "./preview.ts";

const ORIGIN = "https://courses.scottylabs.org";

describe("instructorLinkTarget", () => {
  it("reads instructor pages, with the link's semester", () => {
    expect(instructorLinkTarget("/instructors/david-kosbie?term=S27", ORIGIN)).toEqual({
      slug: "david-kosbie",
      name: null,
      termId: "S27",
    });
    expect(instructorLinkTarget("/instructors/john-smith-2", ORIGIN)).toEqual({
      slug: "john-smith-2",
      name: null,
      termId: null,
    });
    expect(
      instructorLinkTarget(`${ORIGIN}/instructors/david-kosbie/?term=all&q=kos`, ORIGIN),
    ).toEqual({
      slug: "david-kosbie",
      name: null,
      termId: "all",
    });
    expect(instructorLinkTarget("/instructors/jos%C3%A9-p%C3%A9rez?term=X", ORIGIN)).toEqual({
      slug: "jos\u00E9-p\u00E9rez",
      name: null,
      termId: null,
    });
  });

  it("ignores the list, other pages and other sites", () => {
    expect(instructorLinkTarget("/instructors?term=S27", ORIGIN)).toBeNull();
    expect(instructorLinkTarget("/instructors/", ORIGIN)).toBeNull();
    expect(instructorLinkTarget("/instructors/a/b", ORIGIN)).toBeNull();
    expect(instructorLinkTarget("/course/15-122?term=S27", ORIGIN)).toBeNull();
    expect(instructorLinkTarget("https://example.com/instructors/david-kosbie", ORIGIN)).toBeNull();
    expect(instructorLinkTarget("/instructors/%E0%A4%A", ORIGIN)).toBeNull();
    expect(instructorLinkTarget("http://[bad", ORIGIN)).toBeNull();
  });

  it("leaves course links to course previews, and the other way round", () => {
    expect(linkTarget("/instructors/david-kosbie?term=S27", ORIGIN)).toBeNull();
    expect(instructorLinkTarget("/?term=S27&course=15-112", ORIGIN)).toBeNull();
  });
});

describe("instructorPreview", () => {
  it("marks names, with a semester when one is given", () => {
    expect(instructorPreview("David Kosbie")).toEqual({
      "data-instructor-preview": "David Kosbie",
    });
    expect(instructorPreview(" David Kosbie ", "S27")).toEqual({
      "data-instructor-preview": "David Kosbie",
      "data-instructor-term": "S27",
    });
    expect(instructorPreview("David Kosbie", "all")).toEqual({
      "data-instructor-preview": "David Kosbie",
      "data-instructor-term": "all",
    });
    expect(instructorPreview("David Kosbie", "2027")).toEqual({
      "data-instructor-preview": "David Kosbie",
    });
  });

  it("marks nothing without a name", () => {
    expect(instructorPreview(null)).toEqual({});
    expect(instructorPreview("  ")).toEqual({});
    expect(instructorPreview("off")).toEqual({});
    expect(instructorPreview("--")).toEqual({});
  });
});

describe("markedInstructor", () => {
  it("reads a marked name, and nothing for off", () => {
    expect(markedInstructor("David Kosbie", "F26")).toEqual({
      slug: "david-kosbie",
      name: "David Kosbie",
      termId: "F26",
    });
    expect(markedInstructor("David Kosbie", null)).toEqual({
      slug: "david-kosbie",
      name: "David Kosbie",
      termId: null,
    });
    expect(markedInstructor("off", null)).toBeNull();
    expect(markedInstructor("", null)).toBeNull();
    expect(markedInstructor(null, "S27")).toBeNull();
  });
});

describe("pageInstructor", () => {
  it("names the instructor the page is about", () => {
    expect(pageInstructor("/instructors/david-kosbie")).toBe("david-kosbie");
    expect(pageInstructor("/instructors/david-kosbie/")).toBe("david-kosbie");
    expect(pageInstructor("/instructors")).toBeNull();
    expect(pageInstructor("/course/15-112")).toBeNull();
  });
});

describe("findInstructor", () => {
  // Made-up people; two share a name, so the second gets a numbered slug.
  const file: InstructorListFile = {
    titles: { "15112": "Fundamentals of Programming" },
    instructors: [
      {
        name: "Ada Example",
        depts: ["CS"],
        courses: ["15112"],
        sections: 2,
        fce: { rating: 4.5, courses: ["15112"] },
      },
      { name: "Sam Sample", depts: ["MSC"], courses: ["21127"], sections: 1 },
      { name: "Sam Sample", depts: ["PHY"], courses: ["33141"], sections: 1 },
    ],
  };
  const directory = instructorDirectory(file);

  it("finds links by slug and marked names by name", () => {
    expect(findInstructor(directory, { slug: "ada-example", name: null })?.courseIds).toEqual([
      "15112",
    ]);
    expect(findInstructor(directory, { slug: "sam-sample-2", name: null })?.depts).toEqual(["PHY"]);
    expect(
      findInstructor(directory, { slug: "ada-example", name: "Ada Example" })?.fceCourses,
    ).toEqual(["15112"]);
    expect(findInstructor(directory, { slug: "nobody", name: null })).toBeUndefined();
  });
});
