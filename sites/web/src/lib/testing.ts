// The assertion library for unit tests, which run on `deno test` with node:test's describe and it: Jest's `expect`
// from npm. Tests import it from here rather than by name, because the repository's commit hook runs them from the
// repository root, where sites/web's package.json doesn't apply. src/npm.d.ts gives the specifier its types for
// svelte-check and oxlint. (JSR packages can't be used: the Nix build fetches every locked package from npm.)
export { expect } from "npm:expect@30.5.2";
