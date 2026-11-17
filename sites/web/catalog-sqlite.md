# Browser catalog with SQLite WASM

The frontend can query a read-only copy of the public course catalog through
`$lib/catalog-sqlite`. Call the helper from a browser event handler or Svelte's
`onMount`.

```ts
import { queryCatalog } from "$lib/catalog-sqlite";

const terms = await queryCatalog("SELECT * FROM catalog_terms LIMIT ?", [10]);
```

The helper returns rows as objects. Keep SQL in application code and pass user
input as bound parameters. Components can handle a rejected query like any
other failed data request.

## Loading and caching

The root layout starts loading the public snapshot and checking session status
in parallel. The public request omits cookies. When `/auth/status` reports
`can_access_member_catalog: true`, the app requests `/api/catalog/member-binary`
with the same-origin session cookie. The endpoint checks CMU access again.

The browser decodes gzip via HTTP `Content-Encoding` and opens each SQLite
snapshot read-only in memory. Queries share any preload already in progress and
retry a failed load. Member downloads are canceled and the member connection is
closed when session state changes or the page is left. The server still handles
encryption of deployment artifacts. Browsers receive standard SQLite bytes.

## Follow-up work

Add typed functions such as `searchCourses` above the query helper as screens
need them. Move query execution to a worker if catalog loading or queries
affect UI responsiveness.

## Frontend sign-in and member queries

Import `loggedIn` from `$lib/session` and use `$loggedIn` in Svelte. The
root layout initializes it from the existing session check. Use `$session.status`
when the UI needs to distinguish loading or a failed check from signed out.

`queryCatalog` always queries the public snapshot. For CMU-only data, use
`queryMemberCatalog` and catch `CatalogAccessError`:

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

A confirmed sign-in without CMU access throws `cmu-access-required`.
A failed member download throws `member-catalog-unavailable`. Treat the
frontend errors as UI feedback. The server enforces access to the member bytes.
