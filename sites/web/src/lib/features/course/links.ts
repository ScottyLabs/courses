// Where course links inside a course view go. The full page links to other course pages; the Courses page's detail
// panel opens them in the panel instead. Set once by the view, read by its sections.
import { getContext, setContext } from "svelte";
import { coursePath } from "$lib/core/routes";

/** A link to a course (five-digit id) in a semester; null means no particular semester (the all-semesters view). */
export type CourseHref = (courseId: string, termId: string | null) => string;

const KEY = Symbol("course-href");

export const defaultCourseHref: CourseHref = (courseId, termId) => coursePath(courseId, termId);

export function setCourseHref(href: CourseHref): void {
  setContext(KEY, href);
}

/** The enclosing view's course links, or links to course pages. */
export function getCourseHref(): CourseHref {
  return getContext<CourseHref | undefined>(KEY) ?? defaultCourseHref;
}
