// Types for the shared week calendar. Blocks are core/week-grid.ts's CalendarBlock, so pages build them with
// core/blocks.ts (sectionBlocks) or core/planner/format.ts (partBlocks) and hand them in.
export type { CalendarBlock, LaidBlock } from "../../core/week-grid.ts";

/** Something with no set meeting time, listed under the grid ("No set meeting time"). */
export interface TbaItem {
  key: string;
  /** Primary text, usually the course code. */
  label: string;
  /** The course title or another second line. */
  title?: string;
  /** Short note on the right, such as "Lec 1" or "12 units". */
  detail?: string;
  /** Palette index (core/colors.ts). */
  color: number;
  /** Makes the title a link, such as the course page. */
  href?: string;
}

/** Day by day (phones) or the whole week. */
export type CalendarView = "day" | "week";
