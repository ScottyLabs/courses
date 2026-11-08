# CMU Courses

Course browsing app with a Svelte frontend and Rust API.

## Maintainers

Start with the [maintainer guide](sites/docs/src/maintaining.md), [feature matrix](sites/docs/src/features.md), and [deployment runbook](sites/docs/src/deployment.md).

## Run locally

1. Install [Lix](https://lix.systems/install/) with the ScottyLabs binary cache:

   ```sh
   curl -sSf -L https://install.lix.systems/lix | sh -s -- install \
     --extra-conf "extra-substituters = https://scottylabs.cachix.org" \
     --extra-conf "extra-trusted-public-keys = scottylabs.cachix.org-1:hajjEX5SLi/Y7yYloiXTt2IOr3towcTGRhMh1vu6Tjg="
   ```

   Then log in with `nix run git+https://git.cmu.dev/ScottyLabs/kennel#login`. The development shell loads the OIDC settings and other secrets. You need access to those secrets for CMU sign-in; there is no local login bypass.

1. Clone the repository and enter the development shell:

   ```sh
   git clone ssh://forgejo@git.cmu.dev/ScottyLabs/courses.git
   cd courses
   nix develop
   ```

1. Prepare the local catalog if `data/catalog/public.sqlite` and `data/catalog/member.sqlite` are absent. Copy the supplied source exports into `data/raw`, then run:

   ```sh
   python3 scripts/import_catalog.py data/raw
   python3 scripts/verify_catalog.py
   ```

   The catalog and raw exports are intentionally ignored by Git. The local API reads `data/catalog` directly; it does not need CDN or S3 credentials. It creates `data/private/users.sqlite` for local personal data. Keep that file out of Git and static assets. See [backend setup](sites/docs/src/api/sqlite-backend.md) and [data audit](sites/docs/src/api/data-audit.md).

1. Start PostgreSQL, the Ricochet OIDC callback relay, the browser API on port 3002 and the Svelte app on port 5173 from the repository root:

   ```sh
   pitchfork start --local
   ```

1. Open <http://localhost:5173>. Use **Sign in** to test the configured CMU identity provider. The frontend proxies `/api` and `/auth` to the API, and Ricochet returns the callback to port 5173. The API needs `APP_URL=http://localhost:5173` and the callback URL supplied by the development secrets. Stop everything with `pitchfork stop --local`.

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
| `pitchfork logs web-api` | Repository root | Show a local process's output when troubleshooting |

More details: [architecture](sites/docs-migrating-wip/src/overview.md) and
[scraper usage](sites/docs-migrating-wip/src/scraper/running.md).
