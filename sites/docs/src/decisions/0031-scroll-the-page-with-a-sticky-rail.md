# 31. Scroll the page, with a sticky header and rail

Date: 2026-10-24

## Status

Accepted

## Context

The Figma frames show the header and footer fixed to the viewport, with the filter rail and the results scrolling
separately between them. Courseweave also uses a fixed app frame and scrolls panels inside it. A fixed frame keeps 166
pixels of the viewport for the header and footer on every page, including long course lists, and it moves scrolling
off the window: the browser's scroll restoration, find in page, pull to refresh and the phone address bar all work
less well when the document itself doesn't scroll.

## Decision

The window scrolls. The header is sticky at the top, and the filter and profile rails are sticky columns below it that
scroll on their own when they are taller than the viewport. The footer follows the page content. At phone widths the
rail is a sheet and the header shrinks to 64 pixels.

## Consequences

- The footer is below the fold on rail pages instead of always visible as in the Figma frames
- SvelteKit's scroll restoration and anchor links work on every page
- Anything that needs to stay in view while the page scrolls, such as a schedule's totals, should be `sticky` with
  `top-(--header-h)`, the header's height at the current breakpoint
