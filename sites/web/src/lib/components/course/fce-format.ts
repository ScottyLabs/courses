// Number formats for FCE figures in stat boxes and tables.

/** Shown where a figure is missing: an en dash. */
export const NO_VALUE = String.fromCodePoint(0x2013);

/** Hours a week, to one decimal everywhere FCEs show them (Courseweave's formatHours): 15.2. */
export function hoursText(value: number): string {
  return value.toFixed(1);
}

/** A rating out of 5, to two decimals everywhere FCEs show them (Courseweave's formatRating): 4.50. */
export function ratingText(value: number): string {
  return value.toFixed(2);
}
