# CMU Courses

Course browsing app with a Svelte frontend and Rust API.

## Maintainers

Start with the [maintainer guide](sites/docs/src/maintaining.md), [feature matrix](sites/docs/src/features.md), and [deployment runbook](sites/docs/src/deployment.md).

## Run locally

1. Install Lix and configure the ScottyLabs binary cache using the [Kennel setup guide](https://docs.kennel.scottylabs.org/guides/local-development/#installing-nix). To use Collar from outside the project shell, install it once:

   ```sh
   nix profile install 'git+https://git.cmu.dev/ScottyLabs/collar#collar'
   ```

1. Clone the repository and enter its shell with Collar:

   ```sh
   git clone ssh://forgejo@git.cmu.dev/ScottyLabs/courses.git
   cd courses
   collar auth bao login
   collar dev shell
   ```

   From an existing checkout, pull the latest changes and run `collar dev shell` from the repository root. The shell also bundles Collar, so `nix develop` is available as a bootstrap option. OpenBao supplies the app's OIDC settings to members of the [CMU Courses team](https://git.cmu.dev/ScottyLabs/governance#joining-a-team). Use `collar auth login` for Forgejo access.

1. Prepare the local catalog if `data/catalog/public.sqlite` and `data/catalog/member.sqlite` are absent. Place the Stellic scraper output under `exported/`, then run:

   ```sh
   cargo run --release -p courses-index -- --import-catalog exported
   python3 scripts/verify_catalog.py
   ```

   The catalog and scraper exports are intentionally ignored by Git. The local API reads `data/catalog` directly; it does not need CDN or S3 credentials. It creates `data/private/users.sqlite` for local personal data. Keep that file out of Git and static assets. See [backend setup](sites/docs/src/api/sqlite-backend.md) and [data audit](sites/docs/src/api/data-audit.md).

1. Start PostgreSQL, the Ricochet OIDC callback relay, the browser API on port 3002 and the Svelte app on port 5173 from the repository root:

   ```sh
   collar dev up
   ```

1. Open <http://localhost:5173>. Use **Sign in** to test the configured CMU identity provider. The frontend proxies `/api` and `/auth` to the API, and Ricochet returns the callback to port 5173. The API needs `APP_URL=http://localhost:5173` and the callback URL supplied by the development secrets. Use `collar dev status` to inspect the processes, `collar dev logs web-api` for API logs, and `collar dev down` to stop the stack.

If the API cannot start, first check that the catalog exists and that the development shell loaded the OIDC settings. A `403` from `/api/me` after login means the provider did not supply a verified CMU email in the ID token or matching UserInfo response. The header uses `GET /auth/status`, which returns `200` with `name: null` while signed out and the account name after sign-in. Its `can_access_member_catalog` field controls whether the frontend preloads the member SQLite snapshot. Member-only endpoints such as `/api/me` still return `401` before login.

## Preview a production build

From `sites/web`, with the development environment loaded:

```sh
deno task build
deno task preview
```

Open <http://localhost:4173> to inspect the built UI. Vite's preview server does not use the development `/api` and `/auth` proxy, so use the development server above to test sign-in. Rebuild after changes. Deployment serves `sites/web/build` through `courses-web-api` using `STATIC_DIR`.

## Useful commands

| Command | Run from | Purpose |
| -- | -- | -- |
| `deno task check` | `sites/web` | Check Svelte and TypeScript |
| `cargo test -p courses-web-api` | Repository root | Run API unit tests |

More details: [architecture](sites/docs-migrating-wip/src/overview.md) and [scraper usage](sites/docs-migrating-wip/src/scraper/running.md).
