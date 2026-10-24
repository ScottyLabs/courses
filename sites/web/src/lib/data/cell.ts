// A single reactive value. The stores and loaders keep their state in cells so their logic is plain TypeScript that
// `deno test` runs as is; cell.svelte.ts makes cells out of Svelte's $state for the website, and plainCell is a bare
// variable for tests and scripts. Values are replaced, never mutated in place.

export interface Cell<T> {
  value: T;
}

export type CellFactory = <T>(initial: T) => Cell<T>;

/** A cell with no reactivity: for tests and server-side code. */
export const plainCell: CellFactory = <T>(initial: T): Cell<T> => ({ value: initial });
