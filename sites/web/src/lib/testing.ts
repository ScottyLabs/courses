// The assertion library for unit tests, which run on `deno test` with node:test's describe and it. Tests import it
// from here rather than by name, because the repository's commit hook runs them from the repository root, where
// sites/web's package.json doesn't apply. src/jsr.d.ts gives the specifier its types for svelte-check and oxlint.
export { expect } from "jsr:@std/expect@1.0.20";
