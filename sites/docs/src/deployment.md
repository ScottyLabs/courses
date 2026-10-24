# Deployment runbook

> **This branch can't deploy until a new catalog release is published.** The web API now reads one `catalog.sqlite`
> ([decision 15](decisions/0015-build-the-catalog-in-rust.md)), but the committed `deployments/catalog.json` still
> pins the old six-file release (`public.sqlite`, `member.sqlite` and the browser files). Until a new manifest is
> committed, startup in a managed deployment fails with "deployments/catalog.json pins an old-format release" and the
> health check never passes. To unblock it:
>
> 1. Build the catalog with `courses-index build`.
> 1. Publish it as a single encrypted `catalog.sqlite` with `schema_version` in the manifest. `scripts/publish_catalog.py`
>    still uploads the six old files and must be updated first; that change is outside the web API work.
> 1. Commit the new `deployments/catalog.json` with the code.

## Current production finding

The pushed backend commit `d220aa7` built successfully in Kennel but its `web-api` deployment failed the startup health check. The old code tried to create a relative personal database in a read-only service filesystem, and the generated catalog was absent from the build. A successful Nix build is not a successful rollout.

A production-only encryption key and catalog release have been provisioned for the old format. Staging/preview keys have not been provisioned.

The configuration enables Kennel SQLite, uses its `DATABASE_PATH`, and serves the frontend and API from the `web-api` service at `cmu.courses`. Do not also declare a static site on that domain: the SPA uses same-origin `/api` and `/auth` routes. The historical `api.cmu.courses` service remains separate.

## Required runtime configuration

| Setting | Source and purpose |
| --- | --- |
| `PORT`, `APP_URL`, `COMMIT_HASH` | Injected by Kennel; `APP_URL` must be the exact public origin |
| `DATABASE_PATH` | Kennel SQLite resource; persistent per-service database. The pinned catalog is installed in `catalogs/` beside it |
| `USER_DB_PATH` | Optional explicit personal database override |
| `CATALOG_DIR` | Optional directory holding a `catalog.sqlite` to serve instead of the pinned release |
| `CATALOG_ENCRYPTION_KEY` | Dedicated base64-encoded 32-byte AES key in the deployment secret store |
| `KEYCLOAK_URL`, `KEYCLOAK_REALM` | Trusted university identity issuer |
| `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET` | Existing OIDC client |
| `OAUTH_RELAY_URL` | Existing Ricochet callback relay |
| `PROJECT_ADMIN_GROUP` | Optional administrative group; never grants CMU membership |
| `STATIC_DIR` | Set by the Nix package wrapper to the built Svelte site; enables server-rendered course head tags |
| `SYLLABI_BUCKET`, `SYLLABI_PREFIX` | Private bucket and key prefix (default `syllabi/`) holding syllabus files |
| `SYLLABI_S3_ENDPOINT`, `SYLLABI_S3_REGION` | Default `https://s3.scottylabs.org` and `us-east-1` |
| `SYLLABI_ACCESS_KEY_ID`, `SYLLABI_SECRET_ACCESS_KEY` | Read-only keys for the syllabus bucket; required with `SYLLABI_BUCKET` |

Without syllabus storage the service still starts and `/api/syllabi/*` answers 503. `DEV_USER` has no effect in the
Nix build, which never enables the `dev-auth` feature. The [web API reference](api/web-api.md) lists every setting.

The catalog encryption key and syllabus keys are server-only. Do not put them in a Vite/build scope, Git, shell history or logs. The `catalog_publish` scope exposes only the catalog key and CDN credentials to the publishing process. The key must be available to each environment consuming the pinned release. Provisioning new secrets requires an authorized operator.

### Syllabi

Syllabus files are served only to signed-in CMU users ([ADR 24](decisions/0024-serve-syllabi-from-private-object-storage.md)). The scraper currently uploads them to the `cdn-courses` bucket under `syllabi/`, which is publicly readable, so anyone with a key can fetch a file from the CDN without signing in. The gate only holds once syllabi are copied to a private bucket (or a private prefix the CDN doesn't serve) and `SYLLABI_BUCKET` points at it with read-only keys.

## Release procedure

1. Build a new catalog with `courses-index build`. Review its data report and source dates.

1. Run backend, frontend and catalog checks described in the [maintainer guide](maintaining.md).

1. With a dedicated encryption key provisioned, install publisher dependencies in a virtual environment: `pip install -r scripts/publish-requirements.txt`.

1. Run the publisher from the repository root with the catalog at `data/catalog/catalog.sqlite`:

   ```sh
   secretspec --reason 'Publish verified CMU Courses catalog' run \
     -p openbao -P prod -S catalog_publish -- python scripts/publish_catalog.py
   ```

1. Review `deployments/catalog.json`. It pins one immutable encrypted object and its hashes, not plaintext data or secrets:

   ```json
   {
     "id": "<release uuid>",
     "schema_version": 2,
     "files": {
       "catalog.sqlite": {
         "url": "https://cdn.scottylabs.org/courses/sqlite-releases/<id>/catalog.sqlite.enc",
         "sha256": "<ciphertext sha256>",
         "bytes": 0,
         "plain_sha256": "<catalog.sqlite sha256>",
         "plain_bytes": 0
       }
     }
   }
   ```

   The object is a 12-byte nonce followed by the AES-256-GCM sealed gzip of `catalog.sqlite`. `schema_version` must equal the schema this build reads (`courses_catalog::SCHEMA_VERSION`); the service refuses any other. Commit the manifest together with code changes. The old release remains available for rollback.

1. Push to `main` to trigger production, `staging` for staging, or use a pull request for a preview. Do not force-push unrelated upstream changes. Kennel resolves secrets, builds the Nix packages, starts the service, then checks `/api/health` before routing traffic.

1. Check **both** `kennel/build` and `kennel/deploy` statuses on the pushed SHA. Investigate failed CI checks separately; do not call a failed status a completed deployment.

1. Check anonymous health, `/api/terms`, a public catalog document, a member-only document (401 signed out) and `/course/15-122` on the deployed site. Then perform a real CMU sign-in, save and reload a schedule, restart or redeploy and confirm you're still signed in, sign out, and confirm restricted data is unavailable.

First startup installs the pinned encrypted catalog into `catalogs/<release-id>/catalog.sqlite` under the persistent SQLite state directory. It checks the ciphertext and plaintext hashes, opens the file to check its schema, and only then marks the generation complete. Download or decryption failure leaves no completed generation. Keep network transfer and installation within Kennel's startup grace period; if the release grows too large, move artifact installation into an infrastructure pre-start step rather than reporting health early.

## Health

`/api/health` is 200 only when the catalog opens with this build's schema version and a write to the personal database commits. A missing, wrong-schema or empty catalog stops startup with an error naming the file rather than starting unhealthy.

## Sessions

Sessions are stored in the personal database's `sessions` table ([ADR 23](decisions/0023-persist-sessions-in-the-user-database.md)), so restarts and deploys don't sign anyone out. Rows are keyed by a SHA-256 of the session id, so the database can't be replayed as cookies, but the rows hold users' OIDC access and refresh tokens. Expired sessions are deleted hourly. Keycloak's token lifetimes still bound how long a session stays signed in. Deleting an account removes all of its sessions.

## Backup and rollback

Back up the **personal database** using SQLite's online backup API. Copying only the main file while WAL writes are active can lose data. The database holds personal records and OIDC refresh tokens, so restrict backups to authorized operators and test restoration into an isolated path.

```sh
python3 scripts/backup_users.py /path/to/users.sqlite /private/backups/users-YYYYMMDD.sqlite
```

Catalogs are reproducible and pinned; personal data is not. Keep the key with the approved secret-management system so catalog releases remain decryptable. Retain previous encrypted releases while any deployed commit references them.

For application/catalog rollback, redeploy a known-good Git commit and its release manifest. **Do not restore an old personal database as an application rollback**: that would discard subsequent user writes. Schema changes are additive and versioned by `PRAGMA user_version` (currently 4); a build refuses to open a database newer than it knows, so rolling back across a schema change needs a forward-compatible build or an explicit restore procedure. Future destructive migrations require an explicit restore/forward-migration procedure.

## Diagnosing startup failure

- "pins an old-format release": `deployments/catalog.json` still names the six-file release; publish and commit a `catalog.sqlite` release (see the top of this page).
- "pins catalog schema N, but this build reads schema M": publish a catalog built by the matching `courses-index`.
- "Read-only filesystem": verify `scottylabs.sqlite.enable`, `DATABASE_PATH`, and the service's persistent directory.
- "CATALOG_ENCRYPTION_KEY is required" or "catalog authentication failed": verify the release key in the correct profile; never print it.
- Checksum mismatch: verify the committed manifest and immutable CDN object. Do not skip verification.
- "the user database has schema N, newer than this build's": a newer build migrated the database; deploy a build at least that new.
- "SYLLABI_ACCESS_KEY_ID must be set with SYLLABI_BUCKET": provide the syllabus keys or unset the bucket.
- OIDC discovery failure: check issuer URL, realm, connectivity and certificates.
- Login succeeds but `/api/me` returns 403: inspect the issuer's verified CMU email mapping, not project-group membership.
- Anonymous API works but login or writes fail with 403 `forbidden`: confirm frontend and API are on the same origin and `APP_URL` matches it exactly.
