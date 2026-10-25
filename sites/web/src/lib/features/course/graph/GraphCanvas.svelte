<script lang="ts">
  // The requisite graph's scroll box: the placed graph (./layout.ts) scaled by its PanZoom, its lines in an SVG under
  // the courses, labels and buttons. One tab stop: the arrow keys move between courses and buttons by where they're
  // drawn (./navigation.ts), Home goes to the course in the middle, + and - zoom and 0 fits.
  import { Minus, Plus } from "@lucide/svelte";
  import { flushSync, type Snippet } from "svelte";
  import { codeOf } from "$lib/core/requisite-format";
  import type { GraphLayout, Placed, Rect } from "./geometry";
  import GraphLabel from "./GraphLabel.svelte";
  import { isSatisfied, stateLook } from "./looks";
  import { nextInDirection, type Direction } from "./navigation";
  import type { PanZoom } from "./pan-zoom.svelte";
  import type { CourseNode } from "./types";

  let {
    layout,
    panZoom,
    height,
    label,
    describedBy,
    root,
    course,
    ontoggle,
    onmore,
  }: {
    layout: GraphLayout;
    panZoom: PanZoom;
    /** The scroll box's height. */
    height: number;
    /** The graph's accessible name. */
    label: string;
    /** The id of the text that explains the keys. */
    describedBy: string;
    root: Snippet<[number]>;
    course: Snippet<[Extract<Placed, { kind: "course" }>, number]>;
    ontoggle: (node: CourseNode) => void;
    onmore: (group: string) => void;
  } = $props();

  const ROOT = "root";
  const DIRECTIONS: Record<string, Direction | undefined> = {
    ArrowLeft: "left",
    ArrowRight: "right",
    ArrowUp: "up",
    ArrowDown: "down",
  };

  let canvas = $state<HTMLElement>();
  const keyOf = (item: Placed): string => `${item.kind}:${item.node.key}`;
  /** Everything the arrow keys move between: the middle course, the other courses, expand and "more" buttons. */
  const stops = $derived<(Rect & { key: string })[]>([
    { key: ROOT, ...layout.root },
    ...layout.items
      .filter((i) => i.kind === "course" || i.kind === "more" || i.kind === "knob")
      .map((i) => ({ key: keyOf(i), x: i.x, y: i.y, w: i.w, h: i.h })),
  ]);
  let activeKey = $state(ROOT);
  const tabKey = $derived(stops.some((i) => i.key === activeKey) ? activeKey : ROOT);
  const tabindex = (key: string): number => (tabKey === key ? 0 : -1);
  // Satisfied lines go on top, so a met path reads through the brackets it crosses.
  const lines = $derived(layout.lines.toSorted((a, b) => Number(isSatisfied(a.state)) - Number(isSatisfied(b.state))));
  const style = (r: Rect): string => `left:${r.x}px;top:${r.y}px;width:${r.w}px;height:${r.h}px`;

  function focusStop(key: string): void {
    activeKey = key;
    flushSync();
    canvas?.querySelector<HTMLElement>(`[data-graph-key="${CSS.escape(key)}"]`)?.focus({ preventScroll: true });
    const stop = stops.find((i) => i.key === key);
    if (stop !== undefined) panZoom.reveal(stop);
  }

  function zoomKey(key: string): boolean {
    if (key === "+" || key === "=") panZoom.set(panZoom.zoom * 1.25);
    else if (key === "-") panZoom.set(panZoom.zoom / 1.25);
    else if (key === "0") panZoom.fit();
    else return false;
    return true;
  }

  function onkeydown(event: KeyboardEvent): void {
    if (event.altKey || event.ctrlKey || event.metaKey) return;
    panZoom.pristine = false;
    // Arrows and Home move between courses once one has focus; on the scroll box itself they scroll it.
    const onStop = event.target instanceof Element && event.target.closest("[data-graph-key]") !== null;
    const direction = onStop ? DIRECTIONS[event.key] : undefined;
    if (direction !== undefined) {
      const from = stops.find((i) => i.key === tabKey);
      const next = from === undefined ? null : nextInDirection(stops, from, direction);
      if (next !== null) focusStop(next.key);
    } else if (event.key === "Home" && onStop) focusStop(ROOT);
    else if (!zoomKey(event.key)) return;
    event.preventDefault();
  }

  function onfocusin(event: FocusEvent): void {
    const key = (event.target as HTMLElement | null)?.closest<HTMLElement>("[data-graph-key]")?.dataset.graphKey;
    if (key !== undefined) activeKey = key;
  }

  // Listened to here rather than in the markup: the keys act on whichever course inside has focus, and dragging is
  // a mouse convenience over a box the keyboard already scrolls.
  $effect(() => {
    const el = panZoom.viewport;
    if (el === undefined) return;
    const stop = panZoom.attach(el);
    el.addEventListener("keydown", onkeydown);
    el.addEventListener("focusin", onfocusin);
    return () => {
      stop();
      el.removeEventListener("keydown", onkeydown);
      el.removeEventListener("focusin", onfocusin);
    };
  });
</script>

<div
  bind:this={panZoom.viewport}
  bind:clientWidth={panZoom.boxWidth}
  bind:clientHeight={panZoom.boxHeight}
  class="scrollbar-thin relative overflow-auto overscroll-x-contain [touch-action:pan-x_pan-y] {panZoom.dragging
    ? 'cursor-grabbing select-none'
    : 'cursor-grab'}"
  style="height: {height}px"
>
  <div
    class="relative bg-[radial-gradient(var(--color-line)_1px,transparent_1px)] [background-size:20px_20px]"
    style="width: {Math.max(panZoom.boxWidth, layout.width * panZoom.zoom)}px; height: {Math.max(
      panZoom.boxHeight,
      layout.height * panZoom.zoom,
    )}px"
  >
    <div
      bind:this={canvas}
      role="group"
      aria-label={label}
      aria-describedby={describedBy}
      class="absolute origin-top-left"
      style="left: {panZoom.left}px; top: {panZoom.top}px; width: {layout.width}px; height: {layout.height}px; transform: scale({panZoom.zoom})"
    >
      <svg aria-hidden="true" class="pointer-events-none absolute inset-0" width={layout.width} height={layout.height}>
        {#each lines as line, i (i)}
          {@const strong = isSatisfied(line.state)}
          <!-- One-pixel lines sit on half pixels so they stay crisp. -->
          {@const nudge = strong ? 0 : 0.5}
          <line
            x1={line.x1 + (line.x1 === line.x2 ? nudge : 0)}
            y1={line.y1 + (line.y1 === line.y2 ? nudge : 0)}
            x2={line.x2 + (line.x1 === line.x2 ? nudge : 0)}
            y2={line.y2 + (line.y1 === line.y2 ? nudge : 0)}
            class={stateLook(line.state).stroke}
            stroke-width={strong ? 2 : 1}
            stroke-dasharray={line.relation === "coreq" ? "4 3" : undefined}
            stroke-linecap={line.relation === "coreq" ? "butt" : "square"}
          />
        {/each}
      </svg>

      {@render root(tabindex(ROOT))}

      {#each layout.items as item (keyOf(item))}
        {#if item.kind === "course"}
          {@render course(item, tabindex(keyOf(item)))}
        {:else if item.kind === "label"}
          <GraphLabel {item} />
        {:else if item.kind === "knob"}
          <button
            type="button"
            class="absolute grid place-items-center rounded-full border border-line-strong bg-surface text-ink before:absolute before:-inset-2.5 before:content-[''] hover:bg-paper"
            style={style(item)}
            aria-label="{item.node.expanded ? 'Hide' : 'Show'} the prerequisites of {codeOf(item.node.id)}"
            aria-expanded={item.node.expanded}
            title={item.node.shownElsewhere === true ? "Shown where it first appears; show them here too" : undefined}
            data-graph-key={keyOf(item)}
            data-course-preview="off"
            tabindex={tabindex(keyOf(item))}
            onclick={() => ontoggle(item.node)}
          >
            {#if item.node.expanded}<Minus aria-hidden="true" class="size-3" strokeWidth={2.5} />{:else}<Plus
                aria-hidden="true"
                class="size-3"
                strokeWidth={2.5}
              />{/if}
          </button>
        {:else if item.kind === "more"}
          <button
            type="button"
            class="absolute rounded-tag border border-dashed border-line-strong bg-surface px-2 text-label-md text-accent-ink hover:bg-paper"
            style={style(item)}
            aria-expanded={item.node.less}
            data-graph-key={keyOf(item)}
            tabindex={tabindex(keyOf(item))}
            onclick={() => onmore(item.node.group)}>{item.node.label}</button
          >
        {:else}
          <span
            class="absolute flex items-center rounded-tag border border-dashed border-line-strong bg-surface px-2.5 text-body-2 text-ink italic"
            style={style(item)}
            title={item.node.text}><span class="truncate">{item.node.text}</span></span
          >
        {/if}
      {/each}
    </div>
  </div>
</div>
