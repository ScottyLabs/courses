<script lang="ts" module>
  export interface LinePoint {
    /** X label, such as "S26". */
    label: string;
    /** Longer label for the tooltip, such as "Spring 2026". */
    title?: string;
    value: number | null;
    /** Second tooltip line, such as "42 responses". */
    sub?: string;
  }
</script>

<script lang="ts">
  // Courseweave's single-series trend chart (features/fce/LineChart.tsx), for course and instructor FCEs: a snapping
  // crosshair on hover and the arrow keys, nulls breaking the line, the latest value labelled, and an optional
  // reference line.
  let {
    points,
    domain,
    format,
    label,
    height = 132,
    reference,
  }: {
    /** Oldest first. */
    points: readonly LinePoint[];
    domain: [number, number];
    format: (value: number) => string;
    /** Accessible name; the visible title lives outside. */
    label: string;
    height?: number;
    /**
     * A reference, such as the hours a course's units suggest. A number draws a horizontal line; an array (one value
     * per point) draws a stepped line, for references that change over time (a course that became a mini).
     */
    reference?: { value: number | readonly (number | null)[]; label: string };
  } = $props();

  const M = { top: 10, right: 34, bottom: 22, left: 30 };

  let width = $state(0);
  let active = $state<number | null>(null);

  const innerW = $derived(Math.max(0, width - M.left - M.right));
  const innerH = $derived(height - M.top - M.bottom);
  const n = $derived(points.length);
  const lo = $derived(domain[0]);
  const hi = $derived(domain[1]);

  function sx(i: number): number {
    return M.left + (n <= 1 ? innerW / 2 : (i / (n - 1)) * innerW);
  }
  function sy(v: number): number {
    return M.top + innerH - ((Math.min(hi, Math.max(lo, v)) - lo) / (hi - lo || 1)) * innerH;
  }

  // Line segments between consecutive points with values; a missing value breaks the line.
  const segments = $derived.by(() => {
    const out: string[] = [];
    let current: string[] = [];
    for (const [i, p] of points.entries()) {
      if (p.value === null) {
        if (current.length > 1) out.push(current.join(" "));
        current = [];
      } else current.push(`${sx(i)},${sy(p.value)}`);
    }
    if (current.length > 1) out.push(current.join(" "));
    return out;
  });
  const isolated = $derived(
    points
      .map((p, i) => ({ p, i }))
      .filter(
        ({ p, i }) =>
          p.value !== null && (points[i - 1]?.value ?? null) === null && (points[i + 1]?.value ?? null) === null,
      ),
  );
  const ticks = $derived([lo, (lo + hi) / 2, hi]);
  const lastIndex = $derived(points.map((p) => p.value !== null).lastIndexOf(true));
  const lastValue = $derived(lastIndex >= 0 ? (points[lastIndex]?.value ?? null) : null);
  const activePoint = $derived(active === null ? null : (points[active] ?? null));

  const steps = $derived.by(() => {
    if (!reference || typeof reference.value === "number") return [];
    const step = n > 1 ? innerW / (n - 1) : innerW;
    const parts: string[] = [];
    let path = "";
    for (const [i, v] of reference.value.entries()) {
      if (v === null) {
        if (path) parts.push(path);
        path = "";
        continue;
      }
      const x0 = i === 0 ? sx(0) : sx(i) - step / 2;
      const x1 = i === n - 1 ? sx(i) : sx(i) + step / 2;
      path += `${path ? "L" : "M"}${x0},${sy(v)}L${x1},${sy(v)}`;
    }
    if (path) parts.push(path);
    return parts;
  });
  const stepLast = $derived.by(() => {
    if (!reference || typeof reference.value === "number") return null;
    return reference.value.findLast((v) => v !== null) ?? null;
  });

  /**
   * Where the reference's label goes: beside its line, or moved clear of the last value's label when the two would
   * overlap at the right edge.
   */
  function referenceLabelY(y: number): number {
    if (lastValue === null || active !== null) return y;
    const valueY = sy(lastValue);
    return Math.abs(y - valueY) < 12 ? valueY + (y > valueY ? 12 : -12) : y;
  }

  function indexAt(event: PointerEvent): number {
    const target = event.currentTarget;
    if (!(target instanceof Element) || n <= 1) return 0;
    const x = event.clientX - target.getBoundingClientRect().left;
    return Math.max(0, Math.min(n - 1, Math.round(((x - M.left) / innerW) * (n - 1))));
  }

  function onkeydown(event: KeyboardEvent): void {
    if (event.key === "ArrowRight") active = Math.min(n - 1, (active ?? lastIndex) + 1);
    else if (event.key === "ArrowLeft") active = Math.max(0, (active ?? lastIndex + 1) - 1);
    else if (event.key === "Escape") active = null;
    else return;
    event.preventDefault();
  }
</script>

<div bind:clientWidth={width} class="relative">
  {#if width > 0}
    <!-- A chart read point by point with the arrow keys: focusable, with its own keyboard handling. -->
    <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
    <svg
      {width}
      {height}
      role="group"
      aria-label="{label}. Use the arrow keys to read each semester."
      tabindex="0"
      onpointermove={(event) => (active = indexAt(event))}
      onpointerdown={(event) => (active = indexAt(event))}
      onpointerleave={(event) => {
        if (event.pointerType !== "touch") active = null;
      }}
      {onkeydown}
      onblur={() => (active = null)}
      class="block touch-pan-y rounded-tag"
    >
      {#each ticks as t (t)}
        <line x1={M.left} x2={M.left + innerW} y1={sy(t)} y2={sy(t)} class="stroke-line" stroke-width="1" />
        <text x={M.left - 6} y={sy(t)} dy="0.32em" text-anchor="end" class="tabular fill-muted text-[10.5px]">
          {format(t)}
        </text>
      {/each}
      {#if reference && typeof reference.value === "number" && reference.value >= lo && reference.value <= hi}
        <line
          x1={M.left}
          x2={M.left + innerW}
          y1={sy(reference.value)}
          y2={sy(reference.value)}
          class="stroke-muted"
          stroke-width="1"
          stroke-dasharray="4 3"
        />
        <text
          x={M.left + innerW + 4}
          y={referenceLabelY(sy(reference.value))}
          dy="0.32em"
          class="fill-muted text-[10.5px]"
        >
          {reference.label}
        </text>
      {/if}
      {#each steps as d, i (i)}
        <path {d} fill="none" class="stroke-muted" stroke-width="1" stroke-dasharray="4 3" />
      {/each}
      {#if stepLast !== null && reference}
        <text x={sx(n - 1) + 4} y={referenceLabelY(sy(stepLast))} dy="0.32em" class="fill-muted text-[10.5px]"
          >{reference.label}</text
        >
      {/if}
      {#if n > 0}
        <text x={sx(0)} y={height - 6} text-anchor={n > 1 ? "start" : "middle"} class="fill-muted text-[10.5px]">
          {points[0]?.label}
        </text>
        {#if n > 1}
          <text x={sx(n - 1)} y={height - 6} text-anchor="end" class="fill-muted text-[10.5px]">
            {points[n - 1]?.label}
          </text>
        {/if}
      {/if}
      {#each segments as d, i (i)}
        <polyline
          points={d}
          fill="none"
          class="stroke-accent"
          stroke-width="2"
          stroke-linejoin="round"
          stroke-linecap="round"
        />
      {/each}
      {#each isolated as { p, i } (i)}
        <circle cx={sx(i)} cy={sy(p.value ?? 0)} r="3" class="fill-accent" />
      {/each}
      {#if lastValue !== null && active === null}
        <circle cx={sx(lastIndex)} cy={sy(lastValue)} r="4" class="fill-accent stroke-surface" stroke-width="2" />
        <text
          x={sx(lastIndex) + 7}
          y={sy(lastValue)}
          dy="0.32em"
          class="tabular fill-strong text-[11px] font-semibold">{format(lastValue)}</text
        >
      {/if}
      {#if activePoint && active !== null}
        <g pointer-events="none">
          <line x1={sx(active)} x2={sx(active)} y1={M.top} y2={M.top + innerH} class="stroke-muted" stroke-width="1" />
          {#if activePoint.value !== null}
            <circle
              cx={sx(active)}
              cy={sy(activePoint.value)}
              r="4"
              class="fill-accent stroke-surface"
              stroke-width="2"
            />
          {/if}
        </g>
      {/if}
    </svg>
  {:else}
    <div style="height: {height}px"></div>
  {/if}
  {#if activePoint && active !== null}
    <div
      role="tooltip"
      aria-live="polite"
      class="pointer-events-none absolute top-0 z-10 rounded-tag border border-line bg-surface px-2 py-1.5 text-label-md shadow-popover"
      style={sx(active) > width / 2 ? `right: ${width - sx(active) + 8}px` : `left: ${sx(active) + 8}px`}
    >
      <div class="tabular font-semibold text-strong">
        {activePoint.value === null ? "No data" : format(activePoint.value)}
      </div>
      <div class="text-ink">{activePoint.title ?? activePoint.label}</div>
      {#if activePoint.sub}<div class="text-muted">{activePoint.sub}</div>{/if}
    </div>
  {/if}
</div>
