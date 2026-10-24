# CMU Courses

Course search, schedules and degree planning for Carnegie Mellon students: a Svelte website and a Rust API.

## Maintainers

Start with the [maintainer guide](sites/docs/src/maintaining.md), [feature matrix](sites/docs/src/features.md), and [deployment runbook](sites/docs/src/deployment.md).

## Setup

Log in to OpenBao and set up devenv by following kennel's [local development](https://docs.kennel.scottylabs.org/guides/secrets.html#local-development) and [devenv setup](https://docs.kennel.scottylabs.org/guides/deploying.html#2-set-up-devenv) guides. The `dev` profile loads secrets automatically.

The web API reads the catalog from `data/catalog/catalog.sqlite` and stores
personal data (schedules, plans, records, saved courses, friends) in
`data/private/users.sqlite`. See the [web API](sites/docs/src/api/web-api.md).

Build the local catalog from the scraper's output in `exported/` (see
[catalog data](sites/docs/src/api/data.md) for the inputs and how to refresh them):

```sh
cargo run --release -p courses-index -- build
```

It writes `data/catalog/catalog.sqlite` in a few seconds. No S3 credentials are
required for the web API's catalog.

```sh
git clone ssh://forgejo@git.cmu.dev/ScottyLabs/courses.git
cd courses
devenv shell
```

The environment provides Rust and Deno. First startup downloads dependencies.

## Run locally

Use three terminals, each with the development environment loaded. Start from
the repository root in each terminal, after building the catalog (above).

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

### Without Keycloak

To work on the website without OpenBao or Keycloak, run the API with development
sign-in ([decision 26](sites/docs/src/decisions/0026-sign-in-locally-without-keycloak.md)),
which treats every request as a CMU test user:

```sh
APP_URL=http://localhost:5173 DEV_USER=jdoe CATALOG_DIR=data/catalog \
  SYLLABI_DIR=exported/syllabi cargo run -p courses-web-api --features dev-auth
```

`/auth/login?as=<name>` switches to another test user and `/auth/logout` signs
out. Without a catalog or API at all, `deno task dev:mock` in `sites/web` serves
an in-memory mock of the API ([website data and state](sites/docs/src/frontend.md)).

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
| `deno task test` | `sites/web` | Run the website's tests |
| `cargo test --workspace` | Repository root | Run the Rust tests |
| `cargo run --release -p courses-index -- build` | Repository root | Build `data/catalog/catalog.sqlite` |
| `cargo run -p courses-web-api -- openapi` | Repository root | Print the API's OpenAPI document |
| `devenv --no-tui shell` | Repository root | Show plain startup logs when troubleshooting |

More details: [documentation](sites/docs/src/index.md), [architecture decisions](sites/docs/src/decisions/index.md)
and [scraper usage](sites/docs-migrating-wip/src/scraper/running.md).
