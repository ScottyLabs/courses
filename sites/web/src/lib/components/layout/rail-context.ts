// Lets the page toolbar open a RailLayout's rail when, at phone and tablet
// widths, the rail is a sheet instead of a column.
import { getContext, setContext } from "svelte";

export interface RailControls {
  /** True below the desktop breakpoint, where the rail is a sheet. */
  readonly asSheet: boolean;
  /** Active filters, shown on the Filter button. */
  readonly count: number;
  /** Text on the button that opens the sheet, such as "Filter". */
  readonly label: string;
  open: () => void;
}

const RAIL = Symbol("rail");

export function setRail(controls: RailControls): void {
  setContext(RAIL, controls);
}

/** The enclosing RailLayout's controls, or undefined outside one. */
export function getRail(): RailControls | undefined {
  return getContext<RailControls | undefined>(RAIL);
}
