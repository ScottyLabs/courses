# CMU Courses

Course browsing app with a Svelte frontend and Rust API.

## Setup

Install [Nix and devenv](https://devenv.sh/getting-started/). You'll also need
access to `git.cmu.dev` and ScottyLabs' OpenBao secrets service. Ask a maintainer
for access and authentication setup; the `dev` profile loads secrets automatically.

The API also needs a published `catalog.bin` in the configured `CDN_S3_BUCKET`.
If startup reports `NoSuchKey`, ask a maintainer to confirm the catalog location
or publish it using the `Scrape and Publish Catalog` workflow.

```sh
git clone ssh://forgejo@git.cmu.dev/ScottyLabs/courses.git
cd courses
devenv shell
```

The environment provides Rust, Deno, Git, and the WebAssembly tooling. First
startup downloads dependencies and builds the browser's WebAssembly module.

Optional: with [direnv configured in your shell](https://devenv.sh/integrations/direnv/),
run `direnv allow` at the repository root to activate the environment automatically
when entering the directory.

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
| `build-wasm` | Repository root | Rebuild browser bindings after Rust index changes |
| `direnv reload` | Repository root | Reload the environment after configuration changes |
| `devenv --no-tui shell` | Repository root | Show plain startup logs when troubleshooting |

More details: [architecture](sites/docs/src/overview.md) and
[scraper usage](sites/docs/src/scraper/running.md).
