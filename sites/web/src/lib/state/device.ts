// Conveniences remembered on this device, not personal data (decision 17): the last semester visited and, signed
// out, whether the record prompt was answered. Kept in localStorage; every access is guarded, since storage can be
// off or full. The color theme and the cards/list choice are the design system's (components/shell/theme.svelte.ts
// and components/ui/ViewToggle.svelte), kept the same way.
import type { Cell, CellFactory } from "../data/cell.ts";

export interface DeviceState {
  /** The last real semester visited, for leaving the all-semesters view and for pages without a `term`. */
  lastTerm: string | null;
  /** Signed out: the record prompt was answered on this device. */
  onboardingAsked: boolean;
}

const KEY = "cmucourses:device";
const TERM = /^[SMF]\d{2}$/u;

export const DEFAULT_DEVICE: DeviceState = { lastTerm: null, onboardingAsked: false };

/** localStorage, when the browser allows it. */
export function localStore(): Storage | null {
  try {
    return typeof localStorage === "undefined" ? null : localStorage;
  } catch {
    return null;
  }
}

function read(storage: Storage | null): DeviceState {
  try {
    const raw: unknown = JSON.parse(storage?.getItem(KEY) ?? "null");
    if (typeof raw !== "object" || raw === null) return DEFAULT_DEVICE;
    const lastTerm: unknown = Reflect.get(raw, "lastTerm");
    return {
      lastTerm: typeof lastTerm === "string" && TERM.test(lastTerm) ? lastTerm : null,
      onboardingAsked: Reflect.get(raw, "onboardingAsked") === true,
    };
  } catch {
    return DEFAULT_DEVICE;
  }
}

export class DevicePrefs {
  readonly #state: Cell<DeviceState>;
  readonly #storage: () => Storage | null;

  constructor(cell: CellFactory, storage: () => Storage | null = localStore) {
    this.#storage = storage;
    this.#state = cell(read(storage()));
  }

  get lastTerm(): string | null {
    return this.#state.value.lastTerm;
  }

  get onboardingAsked(): boolean {
    return this.#state.value.onboardingAsked;
  }

  /** Remembers a real semester (not "all") as the last one visited. */
  rememberTerm(termId: string): void {
    if (TERM.test(termId) && termId !== this.lastTerm) this.#set({ lastTerm: termId });
  }

  markOnboardingAsked(): void {
    if (!this.onboardingAsked) this.#set({ onboardingAsked: true });
  }

  #set(patch: Partial<DeviceState>): void {
    const next = { ...this.#state.value, ...patch };
    this.#state.value = next;
    try {
      this.#storage()?.setItem(KEY, JSON.stringify(next));
    } catch {
      // Storage off or full: the choice lasts until the page closes.
    }
  }
}
