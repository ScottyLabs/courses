/* oxlint-disable typescript/strict-boolean-expressions, unicorn/explicit-length-check -- Courseweave's code, ported unchanged (decision 14) so fixes carry over between the two codebases */
import type { CatalogEntry } from "../catalog-types.ts";

/** "Usually fall and spring", "Offered intermittently", or null when unknown. */
export function offeredText(entry: CatalogEntry): string | null {
  const seasons = entry.offered ?? [];
  if (entry.intermittent && !seasons.length) return "Offered intermittently";
  if (!seasons.length) return null;
  const names = seasons.map((s) => s[0].toUpperCase() + s.slice(1));
  const list =
    names.length > 1
      ? `${names.slice(0, -1).join(", ")} and ${names.at(-1)!.toLowerCase()}`
      : names[0];
  return `Usually ${list.toLowerCase()}${entry.intermittent ? ", not every year" : ""}`;
}
