// The website's personal data stores and device preferences, reactive through Svelte's $state. Importing this
// module in the browser starts following the session (lib/data).
import { api } from "../api/index.ts";
import { runeCell } from "../data/cell.svelte.ts";
import { catalog, session } from "../data/instances.svelte.ts";
import { DevicePrefs } from "./device.ts";
import { browserEnv } from "./env.ts";
import { Personal } from "./personal.ts";

/** The last semester visited and, signed out, the record prompt's answer: remembered on this device. */
export const device = new DevicePrefs(runeCell);

/** Schedules, plans, record, saved courses and preferences, with their save status. */
export const personal = new Personal({
  api,
  session,
  device,
  cell: runeCell,
  env: browserEnv,
  loadTerm: (id) => catalog.term(id),
});

export const { preferences, schedules, plans, record, saved, links } = personal;

if (typeof window !== "undefined") personal.attach();
