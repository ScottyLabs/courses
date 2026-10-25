// Keyboard and search helpers for dropdown lists (Select, Combobox, MultiSelect).

/** The parts of an `Option` (./types.ts) these helpers read, declared here so their tests run without Svelte. */
interface ListOption {
  readonly value: string;
  readonly label: string;
  readonly keywords?: string;
  readonly disabled?: boolean;
}

function normalize(text: string): string {
  return text.toLowerCase().normalize("NFKD").replaceAll(/\p{M}/gu, "").trim();
}

/** Case- and accent-insensitive match on an option's label, value and keywords. */
export function matchesQuery(option: ListOption, query: string): boolean {
  const needle = normalize(query);
  if (needle === "") return true;
  const haystack = normalize(`${option.label} ${option.value} ${option.keywords ?? ""}`);
  return needle.split(/\s+/u).every((word) => haystack.includes(word));
}

/** Index of the next enabled option from `from` in `direction`, wrapping around. */
export function step(options: readonly ListOption[], from: number, direction: 1 | -1): number {
  const count = options.length;
  if (count === 0) return -1;
  for (let offset = 1; offset <= count; offset += 1) {
    const index = (((from + direction * offset) % count) + count) % count;
    if (options[index]?.disabled !== true) return index;
  }
  return -1;
}

/**
 * Index of the enabled option `distance` steps from `from` in `direction`, stopping at the ends of the list rather
 * than wrapping (arrow keys, Page Up and Page Down in a select). Returns `from` when nothing is enabled that way;
 * `move(options, -1, 1)` is the first enabled option and `move(options, options.length, -1)` the last.
 */
export function move(
  options: readonly ListOption[],
  from: number,
  direction: 1 | -1,
  distance = 1,
): number {
  let target = from;
  let remaining = distance;
  for (
    let index = from + direction;
    index >= 0 && index < options.length && remaining > 0;
    index += direction
  ) {
    if (options[index]?.disabled !== true) {
      target = index;
      remaining -= 1;
    }
  }
  return target;
}

function fold(text: string): string {
  return text.toLowerCase().normalize("NFKD").replaceAll(/\p{M}/gu, "");
}

/**
 * Typeahead in a select: the enabled option whose label starts with `query` (case- and accent-insensitive), searching
 * from the active option `from` and wrapping around. A first letter moves past the active option, so pressing it
 * again goes to the next match; a longer query keeps the active option while it still matches. Repeating one letter
 * ("ccc") cycles through the options starting with it. Returns -1 when nothing matches.
 */
export function typeahead(options: readonly ListOption[], query: string, from: number): number {
  const count = options.length;
  const needle = fold(query);
  if (count === 0 || needle === "") return -1;
  const find = (prefix: string, start: number): number => {
    for (let offset = 0; offset < count; offset += 1) {
      const index = (start + offset) % count;
      const option = options[index];
      if (option !== undefined && option.disabled !== true && fold(option.label).startsWith(prefix))
        return index;
    }
    return -1;
  };
  const after = Math.max(from + 1, 0) % count;
  const found = find(needle, needle.length === 1 ? after : Math.max(from, 0));
  if (found >= 0) return found;
  const repeated = /^(.)\1*$/su.exec(needle)?.[1];
  return repeated === undefined ? -1 : find(repeated, after);
}
