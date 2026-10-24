// The "Aggregate FCEs" choices in the instructor rail: a device convenience like the cards/list switch, so it lives in
// localStorage (design.md, "Conventions") and is shared by the instructor list and the instructor page.
import { readLocal, writeLocal } from "$lib/components/ui/storage";
import { normalizeSettings, type AggregateSettings, type Season } from "./fce-aggregate";

const KEY = "cmucourses:fce-aggregate";

function read(): AggregateSettings {
  try {
    return normalizeSettings(JSON.parse(readLocal(KEY) ?? "null"));
  } catch {
    return normalizeSettings(null);
  }
}

class AggregateChoices {
  #value = $state.raw<AggregateSettings>(read());

  get value(): AggregateSettings {
    return this.#value;
  }

  setSemesters(semesters: number | null): void {
    this.#set({ ...this.#value, semesters });
  }

  setSeason(season: Season, on: boolean): void {
    this.#set({ ...this.#value, seasons: { ...this.#value.seasons, [season]: on } });
  }

  #set(next: AggregateSettings): void {
    this.#value = normalizeSettings(next);
    writeLocal(KEY, JSON.stringify(this.#value));
  }
}

export const aggregateChoices = new AggregateChoices();
