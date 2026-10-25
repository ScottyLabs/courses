# 30. Build UI primitives on native elements and the top layer

Date: 2026-10-24

## Status

Accepted; amended on 2026-10-24: `Select` is a custom select-only combobox rather than a native `<select>` (see
[the amendment](#amendment-custom-selects) below).

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
- Selects and sliders start from native `<select>` and `<input type="range">` where they can (selects no longer do;
  see the amendment). Comboboxes follow the ARIA combobox pattern with `aria-activedescendant`.

## Consequences

- No UI dependency to update or theme; every primitive uses the design tokens directly
- Keyboard and screen reader behavior is ours to maintain, and the dev gallery at `/dev/ui` is where to check it
- Overlays need the Popover API (Chrome 114, Safari 17, Firefox 125); browsers without `@starting-style` show them
  without the entry animation
- A popover shown before a modal dialog opens stays inert under it; panels opened from inside a dialog work normally

## Amendment: custom selects

The browser draws a native select's open list itself: a macOS menu with its own font and check mark, a square list
on Windows, a picker on phones. Next to the site's own dropdowns (the department `MultiSelect`, the requirement
`Combobox`) the semester and sort selects looked like they came from another site, and their closed fields could only
be styled part of the way.

`Select` is now a select-only combobox, the WAI-ARIA pattern: a button with `role="combobox"` that opens a listbox in
the same popover panel, positioning and rows as `Combobox`, with a check mark on the chosen option. Focus stays on the
button and `aria-activedescendant` marks the active option. Enter, Space and the arrow keys open it; the arrows, Home,
End, Page Up, Page Down and typing the start of a label move through it; Enter selects; Escape closes it without
leaving a surrounding dialog; Tab closes it and moves on. Its props stayed the same (`onchange` still reads
`event.currentTarget.value`; `name` submits through a hidden input), and a `trigger` snippet draws it as an icon or
inline text where a transparent native select used to sit over one.

- Every dropdown on the site has the same field, list and keys in every browser and both themes
- Phones get the site's list (40px rows, sized to the screen) instead of the system picker
- The select's keyboard and screen reader behavior is ours to maintain: the keys live in `Select.svelte`, the
  movement and typeahead helpers, with tests, in `listbox.ts`
