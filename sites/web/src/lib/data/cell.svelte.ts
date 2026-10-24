// Cells backed by Svelte's $state.raw: reading `value` in a component, $derived or $effect tracks it, and assigning
// a new value updates them. Raw state, because the stores replace values rather than mutate them.
import type { Cell, CellFactory } from "./cell.ts";

export const runeCell: CellFactory = <T>(initial: T): Cell<T> => {
  let value = $state.raw(initial);
  return {
    get value() {
      return value;
    },
    set value(next: T) {
      value = next;
    },
  };
};
