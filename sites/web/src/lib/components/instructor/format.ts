// Number formats for FCE figures on instructor cards and tables.
import { termName } from "$lib/core/fce-names";

/** Shown where a figure is missing: an en dash. */
export const NO_VALUE = String.fromCodePoint(0x2013);

/** Up to two decimals without trailing zeros: 15.19, 17.1, 4.5. */
export function trimNumber(value: number): string {
  return String(Number(value.toFixed(2)));
}

/** A table cell: the number, or an en dash when there is none. */
export function cellNumber(value: number | null | undefined): string {
  return value === null || value === undefined ? NO_VALUE : trimNumber(value);
}

/** "F24" -> "Fall 2024". */
export function semesterName(term: string): string {
  return termName(term);
}

/** "15112" -> "15-112". */
export function courseCode(courseId: string): string {
  return `${courseId.slice(0, 2)}-${courseId.slice(2)}`;
}
