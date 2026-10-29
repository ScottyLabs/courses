# Deployment runbook

> Backend reset: product endpoints currently return 501. This document describes retained infrastructure or the intended integration contract; feature behavior must be rebuilt. See [backend development](backend-development.md) for current status.

## Current production finding

The pushed backend commit `d220aa7` built successfully in Kennel but its `web-api` deployment failed the startup health check. The old code tried to create a relative personal database in a read-only service filesystem, and the generated catalog was absent from the build. A successful Nix build is not a successful rollout.

A production-only encryption key and catalog release have been provisioned. The same encrypted release was downloaded, decrypted and served in an isolated local bootstrap check using production OIDC discovery; anonymous HTTP/privacy checks passed. A successful remote rollout still requires a pushed SHA with a passing Kennel deployment status. Staging/preview keys have not been provisioned.

The revised configuration enables Kennel SQLite, uses its `DATABASE_PATH`, and serves the frontend and API from the `web-api` service at `cmu.courses`. Do not also declare a static site on that domain: the SPA uses same-origin `/api` and `/auth` routes. The historical `api.cmu.courses` service remains separate.

## Required runtime configuration

| Setting | Source and purpose |
| --- | --- |
| `PORT`, `APP_URL`, `COMMIT_HASH` | Injected by Kennel; APP_URL must be the exact public origin |
| `DATABASE_PATH` | Kennel SQLite resource; persistent per-service database |
| `USER_DB_PATH` | Optional explicit personal database override |
| `CATALOG_DIR` | Optional local/staged catalog directory override |
| `CATALOG_ENCRYPTION_KEY` | Dedicated base64-encoded 32-byte AES key in the deployment secret store |
| `KEYCLOAK_URL`, `KEYCLOAK_REALM` | Trusted university identity issuer |
| `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET` | Existing OIDC client |
| `OAUTH_RELAY_URL` | Existing Ricochet callback relay |
| `PROJECT_ADMIN_GROUP` | Optional administrative group; never grants CMU membership |
| `STATIC_DIR` | Set by the Nix package wrapper to the built Svelte site |

The catalog encryption key is server-only. Do not put it in a Vite/build scope, Git, shell history or logs. The `catalog_publish` scope exposes only the key and CDN credentials to the publishing process. The key must be available to each environment consuming the pinned release. Provisioning new secrets requires an authorized operator.

## Release procedure

1. Import into a new catalog directory and run the verifier. Review the data report and source dates.

1. Run backend, frontend, importer and browser checks described in the [maintainer guide](maintaining.md).

1. With a dedicated encryption key provisioned, install publisher dependencies in a virtual environment: `pip install -r scripts/publish-requirements.txt`.

1. Run the publisher from the repository root with the catalog directory at `data/catalog`:

   ```sh
   secretspec --reason 'Publish verified CMU Courses catalog' run \
     -p openbao -P prod -S catalog_publish -- python scripts/publish_catalog.py
   ```

1. Review `deployments/catalog.json`. It pins immutable encrypted objects and hashes, not plaintext data or secrets. Commit the manifest together with code changes. The old release remains available for rollback.

1. Push to `main` to trigger production, `staging` for staging, or use a pull request for a preview. Do not force-push unrelated upstream changes. Kennel resolves secrets, builds the Nix packages, starts the service, then checks `/api/health` before routing traffic.

1. Check **both** `kennel/build` and `kennel/deploy` statuses on the pushed SHA. Investigate failed CI checks separately; do not call a failed status a completed deployment.

1. Run `python3 scripts/check_deployment.py https://cmu.courses` for anonymous health, S27 data, restricted-route rejection and public catalog transport checks. Then perform a real CMU sign-in, save/reload a schedule, sign out, and confirm restricted data is unavailable.

First startup installs the pinned encrypted catalog into a sibling `catalogs/<release-id>` directory under the persistent SQLite state directory. Download/decryption failure leaves no completed generation. Keep network transfer and installation within Kennel's startup grace period; if the release grows too large, move artifact installation into an infrastructure pre-start step rather than reporting health early.

## Backup and rollback

Back up the **personal database** using SQLite's online backup API. Copying only the main file while WAL writes are active can lose data. Restrict backups to authorized operators and test restoration into an isolated path.

```sh
python3 scripts/backup_users.py /path/to/users.sqlite /private/backups/users-YYYYMMDD.sqlite
```

Catalogs are reproducible and pinned; personal data is not. Keep the key with the approved secret-management system so catalog releases remain decryptable. Retain previous encrypted releases while any deployed commit references them.

For application/catalog rollback, redeploy a known-good Git commit and its release manifest. **Do not restore an old personal database as an application rollback**: that would discard subsequent user writes. Current schema changes are additive; future destructive migrations require an explicit restore/forward-migration procedure.

## Diagnosing startup failure

- "Read-only filesystem": verify `scottylabs.sqlite.enable`, `DATABASE_PATH`, and the service's persistent directory.
- "Catalog key required" or authentication failure: verify the release key in the correct profile; never print it.
- Checksum mismatch: verify the committed manifest and immutable CDN object. Do not skip verification.
- OIDC discovery failure: check issuer URL, realm, connectivity and certificates.
- Login succeeds but `/api/me` returns 403: inspect the issuer's verified CMU email mapping, not project-group membership.
- Anonymous API works but login/POST fails: confirm frontend/API are on the same origin and `APP_URL` matches it exactly.
- Search works initially but local mode fails: inspect the browser's WASM request, catalog version/hash and worker error. The API should remain available as fallback.

Session restart logout is expected with the current in-memory session store. It is not evidence that personal data was lost.
