// Types for the npm modules the tests import by full specifier (src/lib/testing.ts), from the copy package.json
// installs. Deno reads the real module; this only serves svelte-check and oxlint.
declare module "npm:expect@30.5.2" {
  export * from "expect";
}
