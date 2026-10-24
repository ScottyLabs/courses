// Types for the JSR modules the tests import by full specifier (src/lib/testing.ts), from the npm-compatible copy
// that package.json installs. Deno reads the real module; this only serves svelte-check and oxlint.
declare module "jsr:@std/expect@1.0.20" {
  export * from "@std/expect";
}
