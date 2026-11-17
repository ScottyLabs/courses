# Browser catalog with SQLite WASM

Use the typed helpers in `$lib/catalog-sqlite` from a browser event handler or Svelte's `onMount`. For a course list with a "Load more" button:

```ts
import { listCourses, listTerms, type CoursePageOptions } from "$lib/catalog-sqlite";

const terms = (await listTerms()).filter((term) => term.hasCourses);
const options = {
  term: terms[0]?.id,
  search: "systems",
  department: "15",
  sort: { field: "name", direction: "asc" },
  pageSize: 20,
} satisfies CoursePageOptions;

const first = await listCourses(options);
const more = first.nextPage === null
  ? null
  : await listCourses({ ...options, page: first.nextPage });
```

Pages are one-based and contain typed summary rows. The default page size is 25 (valid values: 1 to 100). `nextPage` is null at the end. Omit `term` to browse one row per course code across the catalog. Search matches a literal substring of the code, name, or description, ignores ASCII case, and accepts compact codes like `15122`. Course filters include department name or code prefix, exact level or minimum level, units, and offering season. Ties are broken by course code (or instructor ID) so rows keep a stable order across pages.

Use an async iterator when you want to consume multiple pages without tracking page numbers:

```ts
import { streamCourses } from "$lib/catalog-sqlite";

const controller = new AbortController();
for await (const { items } of streamCourses({ ...options, signal: controller.signal })) {
  appendCourses(items);
  if (!shouldContinue()) break;
}
// Call controller.abort() when the search changes or the view is destroyed.
```

The iterator fetches a page only when requested. Aborting stops further queries and discards pending results; it does not interrupt SQLite while it runs on the main thread or cancel a shared catalog download.

Use `listInstructors` or `streamInstructors` for member-only instructor search. Pass `audience: "member"` to `listCourses` or `streamCourses` to filter by `minFce` or sort by `overallCourse`. Handle a `CatalogAccessError` to show the appropriate sign-in or access message. The raw `queryCatalog` and `queryMemberCatalog` helpers remain available for queries outside this API; pass values as bound parameters.

## Loading and caching

The root layout starts loading the public snapshot and checking session status in parallel. The public snapshot comes from `/api/catalog/binary` with cookies omitted. When `/auth/status` reports `can_access_member_catalog: true`, the app requests `/api/catalog/member-binary` with the same-origin session cookie. The endpoint checks CMU access again.

The browser decodes both downloads via HTTP `Content-Encoding: gzip`. It opens both snapshots read-only in memory. Queries share any preload already in progress and retry a failed load. Member downloads are canceled and the member connection is closed when session state changes or the page is left. The server still handles encryption of deployment artifacts. Browsers receive standard SQLite bytes.

To update the bundled snapshot, regenerate the catalog, run `scripts/verify_catalog.py`, and replace the bundled file with the verified `data/catalog/public-browser.sqlite.gz` before rebuilding.

## Follow-up work

Move query execution to a worker if catalog loading or queries affect UI responsiveness.

## Frontend sign-in and member queries

Import `loggedIn` from `$lib/session` and use `$loggedIn` in Svelte. The root layout initializes it from the existing session check. Use `$session.status` when the UI needs to distinguish loading or a failed check from signed out.

`queryCatalog` always queries the public snapshot. For CMU-only data, use `queryMemberCatalog` and catch `CatalogAccessError`:

```ts
import { CatalogAccessError, queryMemberCatalog } from "$lib/catalog-sqlite";

try {
  const instructors = await queryMemberCatalog("SELECT name FROM catalog_instructors");
} catch (error) {
  if (error instanceof CatalogAccessError && error.code === "sign-in-required") {
    // Show a sign-in prompt.
  } else {
    throw error;
  }
}
```

A confirmed sign-in without CMU access throws `cmu-access-required`. A failed member download throws `member-catalog-unavailable`. Treat the frontend errors as UI feedback. The server enforces access to the member bytes.
