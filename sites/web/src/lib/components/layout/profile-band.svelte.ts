// Lets a profile section put its actions and filters in ProfileLayout's title band (the Figma's Saved Courses band:
// "Department", "Course Level" under the heading). The section declares the snippets in its own markup, so they read
// its state, and hands them over while it's shown.
import { getContext, setContext, type Snippet } from "svelte";

/** What a section shows in the band. */
export interface ProfileBand {
  /** Buttons right of the heading. */
  actions?: Snippet | undefined;
  /** Filters, a count and the cards/list switch under the heading. */
  filters?: Snippet | undefined;
}

class BandSlot {
  current = $state.raw<ProfileBand>({});
}

const BAND = Symbol("profile-band");

/** ProfileLayout: makes the band's slot for the sections inside it. */
export function provideProfileBand(): BandSlot {
  const slot = new BandSlot();
  setContext(BAND, slot);
  return slot;
}

/**
 * A profile section: shows `band()`'s actions and filters in the title band while the section is mounted. Call it
 * while the component initializes.
 */
export function profileBand(band: () => ProfileBand): void {
  const slot = getContext<BandSlot | undefined>(BAND);
  if (slot === undefined) return;
  $effect(() => {
    const mine = band();
    slot.current = mine;
    return () => {
      // The next section may have handed over its band already.
      if (slot.current === mine) slot.current = {};
    };
  });
}

export type { BandSlot };
