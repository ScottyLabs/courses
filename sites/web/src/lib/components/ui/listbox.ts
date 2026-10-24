import type { Option } from "./types";

function normalize(text: string): string {
  return text.toLowerCase().normalize("NFKD").replaceAll(/\p{M}/gu, "").trim();
}

/** Case- and accent-insensitive match on an option's label, value and keywords. */
export function matchesQuery(option: Option, query: string): boolean {
  const needle = normalize(query);
  if (needle === "") return true;
  const haystack = normalize(`${option.label} ${option.value} ${option.keywords ?? ""}`);
  return needle.split(/\s+/u).every((word) => haystack.includes(word));
}

/** Index of the next enabled option from `from` in `direction`, wrapping around. */
export function step(options: readonly Option[], from: number, direction: 1 | -1): number {
  const count = options.length;
  if (count === 0) return -1;
  for (let offset = 1; offset <= count; offset += 1) {
    const index = (((from + direction * offset) % count) + count) % count;
    if (options[index]?.disabled !== true) return index;
  }
  return -1;
}
