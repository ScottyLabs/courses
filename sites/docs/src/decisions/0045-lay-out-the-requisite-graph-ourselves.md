# 45. Lay out the requisite graph ourselves, as HTML in a scroll box

Date: 2026-10-24

## Status

Accepted

## Context

The requisite explorer gets a graph view: the course in the middle, its prerequisite expression to the left drawn as
brackets labelled "all of", "one of" and "2 of" (nested groups as nested brackets, corequisites dashed), what it unlocks to the
right, and the student's standing on every course and bracket. Courses can be expanded into their own requisites,
graphs with dozens of courses need panning and zooming, and phones need a layout that reads at 393px. Every course
on it must stay an ordinary link or button: focusable, previewed by the site's course preview layer
([decision 41](0041-preview-every-course-link.md)), readable by screen readers.

General graph libraries (dagre, ELK, Cytoscape, Svelte Flow) lay out nodes and edges, not boolean expressions: an
OR group would be a node of its own or a crossing of edges, not a bracket its options share, and labels on brackets,
items lined up under them and a parent level with its bracket's middle would be fought for rather than given. ELK is
over a megabyte; canvas renderers draw text the preview layer and screen readers can't reach.

## Decision

- The graph is laid out by our own code (`lib/features/course/graph/`): `model.ts` turns the catalog's requisite AST
  into courses and groups with the student's standing from `evaluateRequisite()` and `checkCourse()` (so it says
  what the course pages say), and `layout.ts` places it as a tidy tree that grows away from the course in the
  middle on both sides: each subtree is measured (height, width, where its line comes in) before it's placed,
  siblings stack, each bracket's label sits level with its items' middle. On phones `outline.ts` places the same
  model top to bottom as an indented outline. Both are pure and unit tested.
- Courses, labels and buttons are absolutely positioned HTML over one SVG of straight lines, all in a canvas scaled
  with a CSS transform inside a native scroll box sized to the scaled graph. Panning is scrolling (wheel, swipe,
  scrollbars, keyboard; a mouse drag on the background scrolls too); zooming changes the scale around the pointer
  and adjusts the scroll. `viewport.ts` has the sums, `pan-zoom.svelte.ts` the input.
- "Expand all" expands each course once, where it first appears; its other appearances keep their + button.

## Consequences

- No dependency, about 1,000 lines of layout and viewport code we own and test, and brackets drawn the way students
  read requisites
- The graph is ordinary DOM: hover and focus previews, focus rings, links that open in new tabs and screen reader
  names work as everywhere else; arrow keys move between courses by where they're drawn
- Lines are only horizontal and vertical, so layouts that would need crossing edges (one course required by two
  groups) repeat the course instead of joining it; "Expand all" expanding each course once keeps that repetition
  small
- Very large graphs are DOM-heavy (a few hundred elements); groups and unlock lists are capped behind "+N more"
