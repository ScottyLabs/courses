// Svelte 5 always accepts null/undefined each-block inputs. The upstream shim
// conditionally imports Svelte to support Svelte 4 too; under Deno's symlinked
// dependencies that ambient import becomes unresolved and erases element types.
// Apply the equivalent Svelte-5-only signature, retaining all type checking.
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { spawnSync } from "node:child_process";
const require = createRequire(import.meta.url);
const root = dirname(require.resolve("svelte-check/package.json"));
const path = join(root, "dist/src/svelte-shims-v4.d.ts");
const old =
  "array: T | (typeof import('svelte') extends { mount: any } ? (undefined | null) : never)";
const replacement = "array: T | undefined | null";
const text = readFileSync(path, "utf8");
if (text.includes(old)) writeFileSync(path, text.replace(old, replacement));
else if (!text.includes(replacement))
  throw new Error("Checker shim changed; review the Svelte 5 each-block workaround.");
const result = spawnSync(
  process.execPath,
  [join(root, "bin/svelte-check"), "--tsconfig", "./tsconfig.json"],
  { stdio: "inherit" },
);
process.exit(result.status ?? 1);
