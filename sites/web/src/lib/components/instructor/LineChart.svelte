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
  // Courseweave's single-series trend chart (features/fce/LineChart.tsx): a snapping crosshair on hover and the
  // arrow keys, nulls breaking the line, the latest value labelled.
  let {
    points,
    domain,
    format,
    label,
    height = 150,
  }: {
    /** Oldest first. */
    points: readonly LinePoint[];
    domain: [number, number];
    format: (value: number) => string;
    /** Accessible name; the visible title lives outside. */
    label: string;
    height?: number;
  } = $props();

  const M = { top: 10, right: 34, bottom: 22, left: 30 };
  let width = $state(0);
  let active = $state<number | null>(null);

  const n = $derived(points.length);
  const innerW = $derived(Math.max(0, width - M.left - M.right));
  const innerH = $derived(height - M.top - M.bottom);
  const lo = $derived(domain[0]);
  const hi = $derived(domain[1]);

  function sx(index: number): number {
    return M.left + (n <= 1 ? innerW / 2 : (index / (n - 1)) * innerW);
  }

  function sy(value: number): number {
    return M.top + innerH - ((Math.min(hi, Math.max(lo, value)) - lo) / (hi - lo || 1)) * innerH;
  }

  const segments = $derived.by(() => {
    const out: string[] = [];
    let current: string[] = [];
    points.forEach((point, index) => {
      if (point.value === null) {
        if (current.length > 1) out.push(current.join(" "));
        current = [];
      } else current.push(`${sx(index)},${sy(point.value)}`);
    });
    if (current.length > 1) out.push(current.join(" "));
    return out;
  });
  const ticks = $derived([lo, (lo + hi) / 2, hi]);
  const lastIndex = $derived(points.map((point) => point.value !== null).lastIndexOf(true));
  const activePoint = $derived(active === null ? null : (points[active] ?? null));

  function isolated(index: number): boolean {
    return (
      points[index]?.value !== null &&
      (points[index - 1]?.value ?? null) === null &&
      (points[index + 1]?.value ?? null) === null
    );
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

<div class="relative" bind:clientWidth={width}>
  {#if width > 0}
    <!-- A focusable chart read with the arrow keys, as in Courseweave; the tooltip below is its live output. -->
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
      {#each ticks as tick (tick)}
        <line x1={M.left} x2={M.left + innerW} y1={sy(tick)} y2={sy(tick)} class="stroke-line" stroke-width="1" />
        <text x={M.left - 6} y={sy(tick)} dy="0.32em" text-anchor="end" class="tabular fill-muted text-[10.5px]">
          {format(tick)}
        </text>
      {/each}
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
      {#each segments as d, index (index)}
        <polyline
          points={d}
          fill="none"
          class="stroke-accent"
          stroke-width="2"
          stroke-linejoin="round"
          stroke-linecap="round"
        />
      {/each}
      {#each points as point, index (index)}
        {#if point.value !== null && isolated(index)}
          <circle cx={sx(index)} cy={sy(point.value)} r="3" class="fill-accent" />
        {/if}
      {/each}
      {#if lastIndex >= 0 && active === null}
        {@const value = points[lastIndex]?.value ?? 0}
        <circle cx={sx(lastIndex)} cy={sy(value)} r="4" class="fill-accent stroke-surface" stroke-width="2" />
        <text x={sx(lastIndex) + 7} y={sy(value)} dy="0.32em" class="tabular fill-ink text-[11px] font-semibold">
          {format(value)}
        </text>
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
    <div style:height="{height}px"></div>
  {/if}
  {#if activePoint && active !== null}
    <div
      role="tooltip"
      aria-live="polite"
      class="pointer-events-none absolute top-0 z-10 rounded-tag border border-line bg-surface px-2 py-1.5 text-label-md font-normal shadow-popover"
      style={sx(active) > width / 2 ? `right: ${width - sx(active) + 8}px` : `left: ${sx(active) + 8}px`}
    >
      <div class="tabular font-semibold text-ink">
        {activePoint.value === null ? "No data" : format(activePoint.value)}
      </div>
      <div class="text-ink">{activePoint.title ?? activePoint.label}</div>
      {#if activePoint.sub}<div class="text-muted">{activePoint.sub}</div>{/if}
    </div>
  {/if}
</div>
