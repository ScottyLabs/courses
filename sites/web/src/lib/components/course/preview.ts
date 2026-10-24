// Course previews (Courseweave's CoursePreview): hovering or focusing a course link, or pressing and holding one on
// a touch screen, shows a card about the course. Every link to a course page, the requisite explorer or the Courses
// page's detail panel gets one; other elements opt in with `coursePreview(id)`, and `data-course-preview="off"` on
// an element turns previews off inside it. CoursePreviewLayer, mounted once in the root layout, does the rest.

/** Marks an element (or, set to "off", a region) for course previews. */
export const PREVIEW_ATTR = "data-course-preview";
/** The semester a marked element previews the course in. */
export const PREVIEW_TERM_ATTR = "data-course-term";

/** Five digits, as course ids are. */
const COURSE_ID = /^\d{5}$/u;
const TERM_ID = /^[A-Z]\d{2}$/u;

/** What a preview shows: a course (five-digit id) in a semester, or in no particular one. */
export interface PreviewTarget {
  courseId: string;
  termId: string | null;
}

function courseIdOf(code: string): string | null {
  const id = code.replaceAll(/\D/gu, "");
  return COURSE_ID.test(id) ? id : null;
}

function termOf(term: string | null | undefined): string | null {
  return typeof term === "string" && TERM_ID.test(term) ? term : null;
}

/** Attributes that give an element a course preview. Accepts "15122" or "15-122"; anything else gives none. */
export function coursePreview(
  courseId: string | null | undefined,
  termId?: string | null,
): Record<string, string> {
  const id = courseIdOf(courseId ?? "");
  if (id === null) return {};
  const term = termOf(termId);
  return term === null ? { [PREVIEW_ATTR]: id } : { [PREVIEW_ATTR]: id, [PREVIEW_TERM_ATTR]: term };
}

/**
 * The course a link previews: `/course/15-122`, `/requisites/15-122` or the Courses page's `/?course=15-122`, with
 * the link's `term` as the semester. Links elsewhere, or off this site, preview nothing.
 */
export function linkTarget(href: string, origin: string): PreviewTarget | null {
  let url: URL;
  try {
    url = new URL(href, origin);
  } catch {
    return null;
  }
  if (url.origin !== origin) return null;
  const term = termOf(url.searchParams.get("term"));
  const match = /^\/(?:course|requisites)\/([^/]+)\/?$/u.exec(url.pathname);
  let code: string | null = null;
  if (match !== null) {
    try {
      code = decodeURIComponent(match[1] ?? "");
    } catch {
      return null;
    }
  } else if (url.pathname === "/") code = url.searchParams.get("course");
  const courseId = code === null ? null : courseIdOf(code);
  return courseId === null ? null : { courseId, termId: term };
}

/** The target of a marked element's attributes, or null for none or "off". */
export function markedTarget(attr: string | null, term: string | null): PreviewTarget | null {
  const courseId = attr === null ? null : courseIdOf(attr);
  return courseId === null ? null : { courseId, termId: termOf(term) };
}

/**
 * The course the page itself is about, so links to it (another semester's sections, "Expand") don't preview what
 * is already on screen: the course page's course, or the course open in the Courses page's panel.
 */
export function pageCourse(pathname: string, search: string): string | null {
  const match = /^\/course\/([^/]+)\/?$/u.exec(pathname);
  if (match !== null) {
    try {
      return courseIdOf(decodeURIComponent(match[1] ?? ""));
    } catch {
      return null;
    }
  }
  if (pathname === "/") return courseIdOf(new URLSearchParams(search).get("course") ?? "");
  return null;
}
