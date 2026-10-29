# Feature status

The backend is intentionally a skeleton. Frontend UI remains present; UI presence does not mean server behavior is implemented.

| Area | Current status |
| --- | --- |
| Server startup, static serving, routing, tracing | Retained |
| OIDC login, logout, verified CMU identity, `/api/me` | Retained; identity lookup has no profile write |
| SQLite schema, catalog installation, import/verification/backup tooling | Retained |
| Health | Retained catalog readiness check; empty catalogs return 503 |
| Catalog reads, search, programs, instructors, browser downloads | 501 skeleton |
| Schedules, saved courses, history, sharing | 501 skeleton |
| Friends, invites, visibility | 501 skeleton |
| Finals reads/import/removal | 501 skeleton |
| Legacy public API | Routes retained; catalog routes return 501 |

See [backend development](backend-development.md) for the work list and route inventory.
