/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../../testing.ts";
import { coursePreview, linkTarget, markedTarget, pageCourse } from "./preview.ts";

const ORIGIN = "https://courses.scottylabs.org";

describe("coursePreview", () => {
  it("marks codes and ids, with a semester when one is given", () => {
    expect(coursePreview("15-122")).toEqual({ "data-course-preview": "15122" });
    expect(coursePreview("15122", "S27")).toEqual({
      "data-course-preview": "15122",
      "data-course-term": "S27",
    });
    expect(coursePreview("15122", "all")).toEqual({ "data-course-preview": "15122" });
  });

  it("marks nothing that isn't a course number", () => {
    expect(coursePreview("15-1")).toEqual({});
    expect(coursePreview(null)).toEqual({});
    expect(coursePreview("")).toEqual({});
  });
});

describe("linkTarget", () => {
  it("reads course pages, the requisite explorer and the Courses page's panel", () => {
    expect(linkTarget("/course/15-122?term=S27", ORIGIN)).toEqual({
      courseId: "15122",
      termId: "S27",
    });
    expect(linkTarget("/course/15122", ORIGIN)).toEqual({ courseId: "15122", termId: null });
    expect(linkTarget("/requisites/21-127?term=F26", ORIGIN)).toEqual({
      courseId: "21127",
      termId: "F26",
    });
    expect(linkTarget("/?term=S27&course=15-213", ORIGIN)).toEqual({
      courseId: "15213",
      termId: "S27",
    });
    expect(linkTarget(`${ORIGIN}/course/15-122?term=all`, ORIGIN)).toEqual({
      courseId: "15122",
      termId: null,
    });
  });

  it("ignores other pages and other sites", () => {
    expect(linkTarget("/schedules?term=S27", ORIGIN)).toBeNull();
    expect(linkTarget("/instructors/smith-john", ORIGIN)).toBeNull();
    expect(linkTarget("/?term=S27", ORIGIN)).toBeNull();
    expect(linkTarget("/course/abc", ORIGIN)).toBeNull();
    expect(linkTarget("https://example.com/course/15-122", ORIGIN)).toBeNull();
    expect(linkTarget("/course/%E0%A4%A", ORIGIN)).toBeNull();
  });
});

describe("markedTarget", () => {
  it("reads a marked element, and nothing for off", () => {
    expect(markedTarget("15122", "S27")).toEqual({ courseId: "15122", termId: "S27" });
    expect(markedTarget("15122", null)).toEqual({ courseId: "15122", termId: null });
    expect(markedTarget("off", null)).toBeNull();
    expect(markedTarget(null, null)).toBeNull();
  });
});

describe("pageCourse", () => {
  it("names the course the page is about", () => {
    expect(pageCourse("/course/15-122", "?term=S27")).toBe("15122");
    expect(pageCourse("/requisites/15-213", "")).toBe("15213");
    expect(pageCourse("/requisites", "")).toBeNull();
    expect(pageCourse("/", "?term=S27&course=15-213")).toBe("15213");
    expect(pageCourse("/", "?term=S27")).toBeNull();
    expect(pageCourse("/plan", "")).toBeNull();
  });
});
