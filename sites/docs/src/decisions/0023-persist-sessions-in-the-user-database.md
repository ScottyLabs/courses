# 23. Persist sessions in the user database

Date: 2026-10-24

## Status

Accepted

## Context

Sessions have lived in `tower-sessions`' in-memory store, so every restart and deploy signed everyone out. With personal
data on the server ([decision 17](0017-store-personal-data-on-the-server.md)), a sign-out in the middle of editing
loses the unsaved change and makes students sign in again for no reason. Deleting an account must also end that
account's sessions on every device, which an in-memory store can't find. The service already has a durable SQLite
database for personal data, on Kennel's persistent storage, used by a single instance.

## Decision

- Sessions are stored in a `sessions` table in the personal database, through a `SessionStore` implementation over
  `rusqlite` that runs on blocking threads like the rest of the database access.
- Rows are keyed by the SHA-256 of the session id, never the id itself, so a copy of the database can't be replayed as
  cookies.
- The first time a member's session is used, it's tagged with the OIDC subject, stored in an `owner` column. Deleting an
  account deletes every session with that owner.
- The cookie, `cmu_courses_session`, is HttpOnly, SameSite=Lax and Secure over https, and expires after 14 days without
  a change. Expired rows are ignored on load and deleted at startup and hourly.
- Keycloak stays in charge of how long a sign-in lasts: the session holds the OIDC tokens, an expired ID token is
  refreshed, and a failed refresh signs the session out.

## Consequences

- Restarts and deploys don't sign anyone out
- The personal database now holds OIDC access and refresh tokens; backups were already restricted, and stay so
- Each signed-in request reads the session from SQLite, and a write follows when the session changes
- Sessions are tied to one database, like the rest of the personal data: running several instances would need shared
  storage for both
