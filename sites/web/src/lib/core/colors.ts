/* oxlint-disable unicorn/prefer-code-point -- Courseweave's code, ported unchanged (decision 14) so fixes carry over between the two codebases */
export const COURSE_COLOR_COUNT = 10;

export const COURSE_COLOR_NAMES = [
  "Rose",
  "Gold",
  "Green",
  "Teal",
  "Blue",
  "Sky",
  "Heather",
  "Moss",
  "Copper",
  "Slate",
];

/** Palette index for any integer, wrapping negatives and overflow. */
export function colorIndex(color: number): number {
  return ((color % COURSE_COLOR_COUNT) + COURSE_COLOR_COUNT) % COURSE_COLOR_COUNT;
}

/** Inline style (`style="..."`) that sets --course for `.course-tint` / `.course-swatch` / `.hatch`. */
export function courseColorStyle(color: number): string {
  return `--course: var(--c${colorIndex(color)})`;
}

/** First palette index not already used, cycling when all are taken. */
export function nextColor(used: number[]): number {
  for (let i = 0; i < COURSE_COLOR_COUNT; i++) if (!used.includes(i)) return i;
  return used.length % COURSE_COLOR_COUNT;
}

/** Stable color for a course id when it isn't in a schedule (e.g. instructor timetables). */
export function hashColor(id: string): number {
  let h = 0;
  for (const ch of id) h = (h * 31 + ch.charCodeAt(0)) >>> 0;
  return h % COURSE_COLOR_COUNT;
}
