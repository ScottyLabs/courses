# 30. Build UI primitives on native elements and the top layer

Date: 2026-10-24

## Status

Accepted

## Context

The website needs dialogs, sheets, popovers, menus, tooltips, searchable selects and sliders that work with the
keyboard, in both themes and at phone widths. Overlays must appear above the filter rail and the toolbar, which scroll
on their own, and above other overlays: a department dropdown opens inside the phone filter sheet. Courseweave used a
few hand-written React components for this; CMU Courses had none. Headless Svelte libraries exist, but each adds a
dependency and its own styling and state conventions that every feature agent would have to learn.

## Decision

We'll build the primitives in `sites/web/src/lib/components/ui` on platform features instead of a component library.

- Dialogs and sheets are native `<dialog>` elements opened with `showModal()`, which makes the rest of the page inert
  and handles Escape. The components add a Tab trap, a scroll lock and returning focus to the opener.
- Popovers, menus, tooltips, dropdown lists and the toast region are `popover="manual"` elements, so they render in
  the top layer above dialogs and outside any scrolling container. A small helper places them next to their anchor
  and flips them when there is no room.
- Selects and sliders start from native `<select>` and `<input type="range">` where they can. Comboboxes follow the
  ARIA combobox pattern with `aria-activedescendant`.

## Consequences

- No UI dependency to update or theme; every primitive uses the design tokens directly
- Keyboard and screen reader behavior is ours to maintain, and the dev gallery at `/dev/ui` is where to check it
- Overlays need the Popover API (Chrome 114, Safari 17, Firefox 125); browsers without `@starting-style` show them
  without the entry animation
- A popover shown before a modal dialog opens stays inert under it; panels opened from inside a dialog work normally
