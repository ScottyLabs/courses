// Where a course page's "Back" goes. The app remembers the last page that wasn't a course page; when that's the
// degree plan, "Back to plan" returns to it (moving between courses from a course page keeps it). A course page
// opened fresh (a new tab, a reload, a shared link) has none and goes back to the course list as usual.
// From Courseweave's state/nav-origin.ts; the app shell calls trackOrigin() after every navigation.

const COURSE_PAGE = /^\/course\/[^/]+\/?$/u;
const PLAN_PAGE = /^\/plan(?:\/|$)/u;

let origin: string | null = null;

/** Records the page being shown (path and query), unless it's a course page. */
export function trackOrigin(pathname: string, search = ""): void {
  if (!COURSE_PAGE.test(pathname)) origin = `${pathname}${search}`;
}

/** The degree plan page a course page was opened from, if that's where the student came from. */
export function planOrigin(): string | null {
  return origin !== null && PLAN_PAGE.test(origin) ? origin : null;
}

/** The last page that wasn't a course page, if any. */
export function lastOrigin(): string | null {
  return origin;
}
