# Feature status

This matrix describes the browser-facing app. “Implemented” means code and a local verification path exist, not that production login and rollout have already been verified. See the [deployment runbook](deployment.md) for the current rollout status.

| Feature | Behavior / implementation | Verification |
| --- | --- | --- |
| Public course search and filters | JSON first, SQLite WASM after warmup; older terms fall back to JSON | SQL adapter tests and real browser smoke test |
| Anonymous privacy | No instructor/FCE/location bytes in public exports; protected routes reject anonymous callers | Full database scan plus API tests |
| CMU authentication | Verified CMU-domain email from configured OIDC issuer | Claim-policy tests; real IdP sign-in remains an operator check |
| Persistent schedules | Owner-scoped create/edit/delete, revision conflicts and term/section checks | Restart, cross-owner, stale revision and invalid-section tests |
| Sharing | Explicit unguessable read-only links, revocation | Share/revoke tests |
| Saved courses | Authenticated persistent stars; anonymous local drafts | Save/reopen/remove tests |
| Taking/taken history | Per-term personal records, profile add/remove UI | Separate history persistence test |
| Friends | Pending requests, recipient acceptance, removal | Consent and unauthorized acceptance tests |
| Invite links | Seven-day, single-use, explicit acceptance, owner revocation | Transactional redemption and repeat-redemption rejection tests |
| Shared courses | Intersection of same-term history, both participants opt in | Authorization enforced in query; no implicit schedule sharing |
| Finals | Private per-term SIO .ics import, reload, removal and export | Timezone/term/duration/parser and owner-isolation tests |
| Major requirements | 204 current public catalog pages plus 2,848 older Stellic audit trees; selection stored with schedule | Imported counts and source provenance |
| Spring 2027 | 3,085 courses / 6,760 sections from CourseWeave | Source converter tests and full/browser database counts |
| Deployment | Persistent SQLite wiring and pinned encrypted catalog bootstrap | Encryption tamper tests; remote rollout must pass Kennel health and live smoke checks |

## Boundaries that should remain visible

- Requirements are readable references, **not an automatic degree-completion evaluator**. Course alternatives, substitutions and double-counting are not certified.
- FCEs are historical course aggregates. Per-semester instructor ratings and respondent drilldowns are not supplied by the current normalized catalog.
- Missing source times, rooms, instructors and descriptions remain missing. Refreshing S27 schedules does not refresh historical FCEs or every description.
- Finals come from the user's own calendar export. There is no university-wide official finals feed in this app.
- The current deployment is one personal SQLite writer with in-memory sessions. Multi-replica deployment and restart-persistent sessions require additional infrastructure.
- “All features implemented” must not be used to imply all historical frontend wishlist items, unprovided datasets, or a validated degree audit. The endpoint contract and this matrix define the delivered behavior.
