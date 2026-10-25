// The week in miniature (WeekThumbnail.svelte): where each meeting goes, as fractions of the grid. It uses the week
// calendar's own maths (core/week-grid.ts) for the days and overlapping meetings, and its hour range with some room
// above the first meeting and below the last.
import type { Day } from "../../core/types.ts";
import { layoutDay, shownDays, type CalendarBlock } from "../../core/week-grid.ts";

export interface ThumbnailRect {
  color: number;
  /** Index of the meeting's day in `days`. */
  column: number;
  /** Overlapping meetings share their column side by side: this one's lane, and how many lanes there are. */
  lane: number;
  lanes: number;
  /** Top edge and height as fractions of the grid's height. */
  top: number;
  height: number;
}

export interface ThumbnailLayout {
  /** Monday to Friday, plus a weekend day when something meets on it. */
  days: Day[];
  /**
   * First and last hour shown: `minHour` to `maxHour`, widened to whole hours that leave at least `padding` minutes
   * before the first meeting and after the last, so none sits on the edge.
   */
  from: number;
  to: number;
  rects: ThumbnailRect[];
}

export function thumbnailLayout(
  blocks: readonly CalendarBlock[],
  minHour = 8,
  maxHour = 18,
  padding = 30,
): ThumbnailLayout {
  const list = [...blocks];
  const days = shownDays(list);
  // The calendar's range (hourRange), with the padding added to each meeting.
  let from = minHour;
  let to = maxHour;
  for (const block of list) {
    from = Math.max(0, Math.min(from, Math.floor((block.start - padding) / 60)));
    to = Math.min(24, Math.max(to, Math.ceil((block.end + padding) / 60)));
  }
  const span = (to - from) * 60;
  const rects = days.flatMap((day, column) =>
    layoutDay(list.filter((block) => block.day === day)).map(({ block, lane, lanes }) => ({
      color: block.color,
      column,
      lane,
      lanes,
      top: (block.start - from * 60) / span,
      height: (block.end - block.start) / span,
    })),
  );
  return { days, from, to, rects };
}
