// Lets the page toolbar open a RailLayout's rail when, at phone and tablet
// widths, the rail is a sheet instead of a column.
import { getContext, setContext } from "svelte";
import type { IconComponent } from "../ui/types";

export interface RailControls {
  /** True below the desktop breakpoint, where the rail is a sheet. */
  readonly asSheet: boolean;
  /** Active filters, shown on the Filter button. */
  readonly count: number;
  /** Text on the button that opens the sheet, such as "Filter". */
  readonly label: string;
  /** The whole button text when it isn't the label and count, such as "Courses (3)". */
  readonly buttonLabel?: string;
  /** The button's icon in place of the filter icon. */
  readonly icon?: IconComponent;
  open: () => void;
  /** Closes the sheet, such as when something in it jumps to a place on the page. */
  close: () => void;
}

/**
 * The open rail's width, shared by RailLayout and ProfileLayout: 340, 380 and 418 pixels up to 1440 wide, growing
 * with the window to 480 at 1920, then faster (30% of each extra pixel) to 672 at 2560 and at most 760.
 */
export const RAIL_WIDTH =
  "w-[340px] xl:w-[380px] min-[90rem]:w-[clamp(418px,calc(235px+12.7vw),480px)] min-[120rem]:w-[clamp(480px,calc(30vw-96px),760px)]";

const RAIL = Symbol("rail");

export function setRail(controls: RailControls): void {
  setContext(RAIL, controls);
}

/** The enclosing RailLayout's controls, or undefined outside one. */
export function getRail(): RailControls | undefined {
  return getContext<RailControls | undefined>(RAIL);
}
