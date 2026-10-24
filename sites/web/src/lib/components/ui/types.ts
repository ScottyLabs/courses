import type { Component } from "svelte";
import type { LucideProps } from "@lucide/svelte";

/** A Lucide icon component, or any component that takes the same props. */
export type IconComponent = Component<LucideProps>;

/** One choice in a Select, Combobox or MultiSelect. */
export interface Option {
  value: string;
  label: string;
  /** A second line under the label in Combobox and MultiSelect lists. */
  description?: string;
  /** Extra text matched by the search box but not shown, such as a department's code. */
  keywords?: string;
  disabled?: boolean;
}
