# Courses API

> Backend reset: product endpoints currently return 501. This document describes retained infrastructure or the intended integration contract; feature behavior must be rebuilt. See [backend development](../backend-development.md) for current status.

The intended public data API is `https://api.cmu.courses/v2`. Course codes,
names, descriptions, units, prerequisites, and similar catalog fields are public.
Section times and locations, instructors, and FCE data require CMU authentication
or a separately issued API key. The browser uses same-origin `/api/*` routes and
the existing OIDC session cookie. See [frontend API contract](./frontend-contract.md)
for the JSON endpoints the redesign expects.

The current `courses-web-api` implementation exposes `/api/me`,
`/api/catalog/version`, and `/api/catalog/binary`. Both catalog routes currently
require an OIDC session, even though the intended public API policy above allows
anonymous access to non-sensitive course fields. Backend work should either add
public JSON catalog routes with field-level redaction or split the binary by
access level. The frontend must never infer authorization from cached data.
