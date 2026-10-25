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
 * The open rail's width, shared by RailLayout and ProfileLayout: 340, 380 and 418 pixels up to 1440 wide, then
 * growing with the window to 560 pixels at 2560.
 */
export const RAIL_WIDTH =
  "w-[340px] xl:w-[380px] min-[90rem]:w-[clamp(418px,calc(235px+12.7vw),560px)]";

const RAIL = Symbol("rail");

export function setRail(controls: RailControls): void {
  setContext(RAIL, controls);
}

/** The enclosing RailLayout's controls, or undefined outside one. */
export function getRail(): RailControls | undefined {
  return getContext<RailControls | undefined>(RAIL);
}
