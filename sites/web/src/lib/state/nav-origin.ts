// Where a course page's "Back" goes. The app remembers the last page that wasn't a course page; when that's the
// degree plan, "Back to plan" returns to it (moving between courses from a course page keeps it), and likewise for a
// schedule, the course list, an instructor or the profile. A course page opened fresh (a new tab, a reload, a shared
// link) has none and goes back to the course list as usual.
// From Courseweave's state/nav-origin.ts; the app shell calls trackOrigin() after every navigation.

const COURSE_PAGE = /^\/course\/[^/]+\/?$/u;
const PLAN_PAGE = /^\/plan(?:[/?#]|$)/u;

let origin: string | null = null;

/** Records the page being shown (path and query), unless it's a course page. */
export function trackOrigin(pathname: string, search = ""): void {
  if (!COURSE_PAGE.test(pathname)) origin = `${pathname}${search}`;
}

/** The degree plan page a course page was opened from, if that's where the student came from. */
export function planOrigin(): string | null {
  return origin !== null && PLAN_PAGE.test(origin) ? origin : null;
}

/** Pages a course page offers to go back to, by path, with the link's label. */
const BACK_LABELS: [RegExp, string][] = [
  [PLAN_PAGE, "Back to plan"],
  [/^\/schedules(?:[/?#]|$)/u, "Back to schedule"],
  [/^\/(?:[?#]|$)/u, "Back to courses"],
  [/^\/instructors\/[^/?#]+/u, "Back to instructor"],
  [/^\/instructors(?:[?#]|$)/u, "Back to instructors"],
  [/^\/profile(?:[/?#]|$)/u, "Back to profile"],
  [/^\/requisites(?:[/?#]|$)/u, "Back to requisites"],
];

/** The course page's way back to the page the student came from, when that's one of the pages above. */
export function originBack(): { href: string; label: string } | null {
  if (origin === null) return null;
  const href = origin;
  const match = BACK_LABELS.find(([pattern]) => pattern.test(href));
  return match === undefined ? null : { href, label: match[1] };
}

/** The last page that wasn't a course page, if any. */
export function lastOrigin(): string | null {
  return origin;
}
