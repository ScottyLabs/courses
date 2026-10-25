<!--
  A week in miniature: a faint grid of day columns and hour rows with each meeting as a small block in its course's
  color, and no text. For lists of schedules, where a full WeekGrid per item would be too heavy: one SVG for the grid
  and a span per meeting. It's decorative, so whatever holds it names what it shows.
-->
<script lang="ts">
  import { courseColorStyle } from "$lib/core/colors";
  import type { CalendarBlock } from "$lib/core/week-grid";
  import { thumbnailLayout, type ThumbnailRect } from "./week-thumbnail";

  let {
    blocks,
    minHour = 8,
    maxHour = 18,
    class: className = "",
  }: {
    /** The meetings to draw (labels and handlers are ignored). */
    blocks: readonly CalendarBlock[];
    /** First and last hour shown; widened to fit every block. */
    minHour?: number;
    maxHour?: number;
    /** Give it a size, such as an aspect ratio. */
    class?: string;
  } = $props();

  const layout = $derived(thumbnailLayout(blocks, minHour, maxHour));
  const columns = $derived(layout.days.length);
  const rows = $derived(layout.to - layout.from);
  // The lines between columns and between hours; the container's own edges are left to whatever frames it.
  const grid = $derived(
    [
      ...Array.from({ length: columns - 1 }, (_, i) => `M${i + 1} 0V${rows}`),
      ...Array.from({ length: rows - 1 }, (_, i) => `M0 ${i + 1}H${columns}`),
    ].join(""),
  );

  const pct = (fraction: number): string => `${(fraction * 100).toFixed(3)}%`;

  function rectStyle(rect: ThumbnailRect): string {
    const left = pct((rect.column + rect.lane / rect.lanes) / columns);
    const width = pct(1 / columns / rect.lanes);
    return `left:calc(${left} + 3px);width:calc(${width} - 6px);top:calc(${pct(rect.top)} + 1px);height:max(calc(${pct(rect.height)} - 2px), 3px);${courseColorStyle(rect.color)}`;
  }
</script>

<div aria-hidden="true" class="relative overflow-hidden {className}">
  <svg
    class="absolute inset-0 size-full text-line"
    viewBox="0 0 {columns} {rows}"
    preserveAspectRatio="none"
    focusable="false"
  >
    <path d={grid} fill="none" stroke="currentColor" stroke-width="1" vector-effect="non-scaling-stroke" />
  </svg>
  {#each layout.rects as rect, index (index)}
    <span class="course-swatch absolute rounded-[3px]" style={rectStyle(rect)}></span>
  {/each}
</div>
