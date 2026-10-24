# CMU Courses

Course browsing app with a Svelte frontend and Rust API.

## Maintainers

Start with the [maintainer guide](sites/docs/src/maintaining.md), [feature matrix](sites/docs/src/features.md), and [deployment runbook](sites/docs/src/deployment.md).

## Setup

Log in to OpenBao and set up devenv by following kennel's [local development](https://docs.kennel.scottylabs.org/guides/secrets.html#local-development) and [devenv setup](https://docs.kennel.scottylabs.org/guides/deploying.html#2-set-up-devenv) guides. The `dev` profile loads secrets automatically.

The browser-facing API reads audience-separated SQLite catalogs from
`data/catalog` and stores personal schedules/saved courses in
`data/private/users.sqlite`. See [backend setup and API](sites/docs/src/api/sqlite-backend.md)
and [data audit](sites/docs/src/api/data-audit.md).

Build the local catalog with `python3 scripts/import_catalog.py data/raw` after
copying the supplied source exports to `data/raw`. No S3 credentials are required
for the web API's catalog.

```sh
git clone ssh://forgejo@git.cmu.dev/ScottyLabs/courses.git
cd courses
devenv shell
```

The environment provides Rust and Deno. First startup downloads dependencies.

## Run locally

Use three terminals, each with the development environment loaded. Start from
the repository root in each terminal.

```sh
# Terminal 1: PostgreSQL and the Ricochet login relay
devenv up
```

```sh
# Terminal 2: web API on port 3002
cargo run -p courses-web-api
```

```sh
# Terminal 3: frontend
cd sites/web
deno task dev
```

Open <http://localhost:5173>. The frontend proxies `/api` and `/auth` to the web
API on port 3002. Stop the foreground processes with Ctrl+C.

## Preview a production build

From `sites/web`, with the development environment loaded:

```sh
deno task build
deno task preview
```

Open <http://localhost:4173>. Keep the API running for backend features. To test
login with the configured callback URL, stop the dev server and use
`deno task preview --port 5173`, with the login relay running. Rebuild after changes.
Preview is for local testing; deployment serves
the generated `sites/web/build` directory through `courses-web-api` using `STATIC_DIR`.

## Useful commands

| Command | Run from | Purpose |
| --- | --- | --- |
| `deno task check` | `sites/web` | Check Svelte and TypeScript |
| `devenv --no-tui shell` | Repository root | Show plain startup logs when troubleshooting |

More details: [architecture](sites/docs-migrating-wip/src/overview.md) and
[scraper usage](sites/docs-migrating-wip/src/scraper/running.md).
