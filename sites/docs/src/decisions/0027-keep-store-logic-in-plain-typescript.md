# 27. Keep the website's store logic in plain TypeScript, with Svelte state injected

Date: 2026-10-24

## Status

Accepted

## Context

The website's data layer is mostly logic: Courseweave's modules (audits, requisites, schedules), and stores that load,
debounce, save, merge after conflicts and keep drafts ([decision 17](0017-store-personal-data-on-the-server.md)). That
logic needs unit tests, and the repository runs TypeScript tests with `deno test`. Svelte 5's runes (`$state`,
`$derived`) only work in files the Svelte compiler has processed, which `deno test` doesn't do, so a store written as a
runes class in a `.svelte.ts` file can't be tested there. Courseweave's own tests use Vitest, and the repository's
commit hook runs `deno test` from the repository root, where `sites/web`'s import map and package.json don't apply.

## Decision

- Stores and loaders are plain TypeScript classes. They keep every piece of reactive state in a `Cell`, a single value
  replaced on change. The website passes `runeCell`, a cell backed by `$state.raw` in a `.svelte.ts` file; tests pass
  `plainCell`, a bare variable. Only `instances.svelte.ts` files create the website's instances.
- The pure half of Courseweave's stores, the record types and the edits each action makes, lives in
  `src/lib/core/state` beside the rest of Courseweave's modules; the classes in `src/lib/state` apply those edits and
  save the results.
- Tests use `node:test`'s `describe` and `it`, which Deno runs natively, and `expect` from `@std/expect` (`@std/testing`'s
  BDD module is deprecated in favor of `node:test`). Modules import each other with `.ts` extensions, and tests import
  `expect` through `src/lib/testing.ts` by its `jsr:` specifier, so the hook's run from the repository root resolves
  everything. `package.json` installs the same package for svelte-check and oxlint, and `src/jsr.d.ts` maps the
  specifier to it.

## Consequences

- The stores' tests run the same code the website runs, with a fake clock and the mock API
  ([decision 29](0029-develop-against-an-in-memory-mock-api.md)) instead of a browser
- Reactivity rests on cells: a store that kept state in a plain field would silently stop updating the page, so new
  state goes in a cell
- Values in cells are replaced, never mutated, which also lets components compare them by reference
- Ported Courseweave tests changed only their imports and the few Vitest-only calls (`describe.skipIf`, per-test
  timeouts)
