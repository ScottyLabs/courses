/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../testing.ts";
import { originBack, trackOrigin } from "./nav-origin.ts";

/** Visits a page, then two course pages from it, and reads the way back. */
function back(path: string, search = ""): ReturnType<typeof originBack> {
  trackOrigin(path, search);
  trackOrigin("/course/15-122", "?term=S27");
  trackOrigin("/course/15-213", "?term=S27");
  return originBack();
}

describe("originBack", () => {
  it("goes back to the page a course page was opened from", () => {
    expect(back("/plan", "?plan=p1")).toEqual({ href: "/plan?plan=p1", label: "Back to plan" });
    expect(back("/schedules", "?term=S27&schedule=s1")).toEqual({
      href: "/schedules?term=S27&schedule=s1",
      label: "Back to schedule",
    });
    expect(back("/", "?term=S27&q=systems")).toEqual({
      href: "/?term=S27&q=systems",
      label: "Back to courses",
    });
    expect(back("/instructors/smith-john", "?term=S27")?.label).toBe("Back to instructor");
    expect(back("/instructors", "?term=S27")?.label).toBe("Back to instructors");
    expect(back("/profile/taken")?.label).toBe("Back to profile");
    expect(back("/requisites/15-122")?.label).toBe("Back to requisites");
  });

  it("has nothing for pages without a way back", () => {
    trackOrigin("/invite/abc");
    expect(originBack()).toBeNull();
    trackOrigin("/finals");
    expect(originBack()).toBeNull();
  });
});
