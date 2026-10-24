# CMU Courses

Course browsing app with a Svelte frontend and Rust API.

## Maintainers

Start with the [maintainer guide](sites/docs/src/maintaining.md), [feature matrix](sites/docs/src/features.md), and [deployment runbook](sites/docs/src/deployment.md).

## Run locally

1. Install and configure [devenv](https://docs.kennel.scottylabs.org/guides/deploying.html#2-set-up-devenv), then [log in to OpenBao](https://docs.kennel.scottylabs.org/guides/secrets.html#local-development). The development shell loads the OIDC settings and other secrets. You need access to those secrets for CMU sign-in; there is no local login bypass.

1. Clone the repository and enter the development shell:

   ```sh
   git clone ssh://forgejo@git.cmu.dev/ScottyLabs/courses.git
   cd courses
   devenv shell
   ```

1. Prepare the local catalog if `data/catalog/public.sqlite` and `data/catalog/member.sqlite` are absent. Copy the supplied source exports into `data/raw`, then run:

   ```sh
   python3 scripts/import_catalog.py data/raw
   python3 scripts/verify_catalog.py
   ```

   The catalog and raw exports are intentionally ignored by Git. The local API reads `data/catalog` directly; it does not need CDN or S3 credentials. It creates `data/private/users.sqlite` for local personal data. Keep that file out of Git and static assets. See [backend setup](sites/docs/src/api/sqlite-backend.md) and [data audit](sites/docs/src/api/data-audit.md).

1. Start each process in its own terminal. Run `devenv shell` from the repository root in each new terminal before the command shown:

   | Terminal | Command | Purpose |
   | --- | --- | --- |
   | 1 | `devenv up` | PostgreSQL and the Ricochet OIDC callback relay |
   | 2 | `cargo run -p courses-web-api` | Browser API on port 3002 |
   | 3 | `cd sites/web && deno task dev` | Svelte app on port 5173 |

1. Open <http://localhost:5173>. Use **Sign in** to test the configured CMU identity provider. The frontend proxies `/api` and `/auth` to the API, and Ricochet returns the callback to port 5173. The API needs `APP_URL=http://localhost:5173` and the callback URL supplied by the development secrets. Stop each process with Ctrl+C.

If the API cannot start, first check that the catalog exists and that the development shell loaded the OIDC settings. A `403` from `/api/me` after login means the provider did not supply a verified CMU email in the ID token or matching UserInfo response. An anonymous `401` from `/api/me` before login is expected.

## Preview a production build

From `sites/web`, with the development environment loaded:

```sh
deno task build
deno task preview
```

Open <http://localhost:4173> to inspect the built UI. Vite's preview server does
not use the development `/api` and `/auth` proxy, so use the development server
above to test sign-in. Rebuild after changes. Deployment serves
`sites/web/build` through `courses-web-api` using `STATIC_DIR`.

## Useful commands

| Command | Run from | Purpose |
| --- | --- | --- |
| `deno task check` | `sites/web` | Check Svelte and TypeScript |
| `deno task test` | `sites/web` | Run frontend unit tests |
| `cargo test -p courses-web-api` | Repository root | Run API unit tests |
| `devenv --no-tui shell` | Repository root | Show plain startup logs when troubleshooting |

More details: [architecture](sites/docs-migrating-wip/src/overview.md) and
[scraper usage](sites/docs-migrating-wip/src/scraper/running.md).
