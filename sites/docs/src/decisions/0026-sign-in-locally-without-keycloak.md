# 26. Sign in locally without Keycloak

Date: 2026-10-24

## Status

Accepted

## Context

Signing in needs Keycloak, an OIDC client secret and the Ricochet relay, all from the secret store. Working on the
website's signed-in features, such as schedules, plans and friends, or testing them in a browser, then needs those
secrets and a real CMU account, and trying friends needs two accounts. Courseweave lets its Worker's dev server stand in
for sign-in. Anything that bypasses sign-in must be impossible to switch on in production.

## Decision

- A `dev-auth` cargo feature, off by default and never enabled by the Nix build, adds development sign-in.
- It's active only when the feature is compiled in, `APP_URL` starts with `http://localhost` (followed by a port, a path
  or nothing), and `DEV_USER` names an Andrew ID. The service then skips OIDC discovery and treats every request as that
  verified CMU user, with subject `dev:<andrew>`, and logs a warning at startup.
- With `DEV_USER` set and any other `APP_URL`, the service refuses to start. Without the feature, `DEV_USER` is ignored
  with a warning.
- `/auth/login?as=<andrew>` acts as another user and `/auth/logout` signs out, through a `cw_dev_user` cookie, so
  signed-out pages and friendships between two users can be tried in one browser. `/auth/login` returns to `DEV_USER`.

## Consequences

- The website can be developed and tested locally with no secrets and no CMU account
- Development users have their own subjects, so their data never mixes with real accounts sharing a database file
- Production builds don't contain the code at all; enabling it needs a source change, not a configuration change
- Behavior that depends on Keycloak (token refresh, group claims, the relay) still needs a real sign-in to test
