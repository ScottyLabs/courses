// How course and instructor cards aggregate FCEs (the rails' "Aggregate FCEs" group): a device convenience, kept in
// localStorage (design.md, "Conventions") and shared by every card and page that shows the aggregate.
import { readLocal, writeLocal } from "$lib/components/ui/storage";
import {
  DEFAULT_FCE_OPTIONS,
  normalizeFceOptions,
  parseFceOptions,
  type FceAggregateOptions,
} from "./fce-aggregate";

const KEY = "cmucourses:fce-aggregate";

class FceOptions {
  #value = $state<FceAggregateOptions>(parseFceOptions(readLocal(KEY)));

  get value(): FceAggregateOptions {
    return this.#value;
  }

  set(next: Partial<FceAggregateOptions>): void {
    this.#value = normalizeFceOptions({ ...this.#value, ...next });
    writeLocal(KEY, JSON.stringify(this.#value));
  }

  reset(): void {
    this.#value = DEFAULT_FCE_OPTIONS;
    writeLocal(KEY, null);
  }
}

export const fceOptions = new FceOptions();
