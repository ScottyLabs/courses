// Instructor previews, the same as course previews (../course/preview.ts): hovering or focusing a link to an
// instructor's page, or pressing and holding one on a touch screen, shows a card about the instructor. Names that
// aren't links opt in with `instructorPreview(name, term)`, and `data-instructor-preview="off"` on a link turns its
// preview off. PreviewLayer (components/preview), mounted once in the root layout, does the rest. Instructors are
// member data, so the layer previews them only while someone is signed in.
import { instructorSlug } from "../../core/data.ts";

/** Marks an element (or, set to "off", a link) for instructor previews; the value is the instructor's name. */
export const INSTRUCTOR_PREVIEW_ATTR = "data-instructor-preview";
/** The semester a marked name previews the instructor in. */
export const INSTRUCTOR_TERM_ATTR = "data-instructor-term";

const TERM_ID = /^[A-Z]\d{2}$/u;
const PAGE = /^\/instructors\/([^/]+)\/?$/u;

/** What an instructor preview shows. */
export interface InstructorTarget {
  /** The instructor page's URL slug ("david-kosbie", "john-smith-2"). */
  slug: string;
  /** Their name as schedules give it, for a marked name; null for a link, which names them by slug. */
  name: string | null;
  /** A semester ("S27"), "all" for all semesters, or null for the one the instructor page would pick. */
  termId: string | null;
}

function termOf(term: string | null | undefined): string | null {
  if (term === "all") return term;
  return typeof term === "string" && TERM_ID.test(term) ? term : null;
}

function slugIn(pathname: string): string | null {
  const match = PAGE.exec(pathname);
  if (match === null) return null;
  try {
    const slug = decodeURIComponent(match[1] ?? "");
    return slug === "" ? null : slug;
  } catch {
    return null;
  }
}

/** Attributes that give a name an instructor preview; a blank name gives none. */
export function instructorPreview(
  name: string | null | undefined,
  termId?: string | null,
): Record<string, string> {
  const trimmed = name?.trim() ?? "";
  if (trimmed === "" || trimmed === "off" || instructorSlug(trimmed) === "") return {};
  const term = termOf(termId);
  return term === null
    ? { [INSTRUCTOR_PREVIEW_ATTR]: trimmed }
    : { [INSTRUCTOR_PREVIEW_ATTR]: trimmed, [INSTRUCTOR_TERM_ATTR]: term };
}

/**
 * The instructor a link previews: `/instructors/<slug>`, with the link's `term` as the semester ("all" included).
 * The instructor list, other pages and other sites preview nothing.
 */
export function instructorLinkTarget(href: string, origin: string): InstructorTarget | null {
  let url: URL;
  try {
    url = new URL(href, origin);
  } catch {
    return null;
  }
  if (url.origin !== origin) return null;
  const slug = slugIn(url.pathname);
  return slug === null ? null : { slug, name: null, termId: termOf(url.searchParams.get("term")) };
}

/** The target of a marked name's attributes, or null for none or "off". */
export function markedInstructor(
  attr: string | null,
  term: string | null,
): InstructorTarget | null {
  const name = attr?.trim() ?? "";
  if (name === "" || name === "off") return null;
  const slug = instructorSlug(name);
  return slug === "" ? null : { slug, name, termId: termOf(term) };
}

/**
 * The instructor the page itself is about (`/instructors/<slug>`), so links to them (another semester's view, "Look
 * in all semesters") don't preview what is already on screen.
 */
export function pageInstructor(pathname: string): string | null {
  return slugIn(pathname);
}
