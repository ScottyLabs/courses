// The requisite graph's device settings (decision 17: conveniences stay on the device): what clicking a course does
// and whether the courses it unlocks show.
import { readLocal, writeLocal } from "$lib/components/ui/storage";

export type ClickAction = "expand" | "preview" | "open" | "center";

export const CLICK_ACTIONS: readonly { value: ClickAction; label: string }[] = [
  { value: "expand", label: "Expand its prerequisites" },
  { value: "preview", label: "Show course preview" },
  { value: "open", label: "Open course page" },
  { value: "center", label: "Center the graph on it" },
];

const CLICK_KEY = "cmucourses:requisite-graph:click";
const UNLOCKS_KEY = "cmucourses:requisite-graph:unlocks";

export function storedClickAction(): ClickAction {
  const stored = readLocal(CLICK_KEY);
  return CLICK_ACTIONS.find((a) => a.value === stored)?.value ?? "expand";
}

export function storeClickAction(action: ClickAction): void {
  writeLocal(CLICK_KEY, action);
}

export function storedShowUnlocks(): boolean {
  return readLocal(UNLOCKS_KEY) !== "off";
}

export function storeShowUnlocks(show: boolean): void {
  writeLocal(UNLOCKS_KEY, show ? null : "off");
}
