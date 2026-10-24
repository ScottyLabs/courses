// Color theme: light, dark or the system's. The choice is a device
// convenience kept in localStorage; app.html applies it before first paint
// using the same key.
import { MediaQuery } from "svelte/reactivity";
import { readLocal, writeLocal } from "../ui/storage";

export type ThemePreference = "light" | "dark" | "system";

export const THEME_KEY = "cmucourses:theme";

function stored(): ThemePreference {
  const value = readLocal(THEME_KEY);
  return value === "light" || value === "dark" ? value : "system";
}

class Theme {
  readonly #systemDark = new MediaQuery("(prefers-color-scheme: dark)", false);

  preference = $state<ThemePreference>(stored());

  /** Whether the page is dark right now. */
  get dark(): boolean {
    return this.preference === "dark" || (this.preference === "system" && this.#systemDark.current);
  }

  set(preference: ThemePreference): void {
    this.preference = preference;
    writeLocal(THEME_KEY, preference === "system" ? null : preference);
  }

  /** Switches to the opposite of what is showing, as an explicit choice. */
  toggle(): void {
    this.set(this.dark ? "light" : "dark");
  }
}

export const theme = new Theme();

/** Mirrors the theme onto <html>. Call once from the root layout inside an effect. */
export function applyTheme(dark: boolean): void {
  document.documentElement.classList.toggle("dark", dark);
}
