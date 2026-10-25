<script lang="ts">
  import { goto } from "$app/navigation";
  import {
    courseIsMini,
    expectedHours,
    expectedHoursText,
    FCE_HOURS_HELP,
    formatHours,
    formatRating,
    workloadRatio,
    workloadTone,
    WORKLOAD_TONE_LABEL,
    type FceSummary,
  } from "$lib/core/fce";
  import type { Course } from "$lib/core/types";

  interface Point {
    course: Course;
    summary: FceSummary;
    units: number;
    /** Minis run half a semester; they're plotted at 2 x units so y = x still means "as its units suggest". */
    mini: boolean;
    /** Weekly hours the units suggest: the x position. */
    unitEq: number;
    hours: number;
    x: number;
    y: number;
    /** Units beyond the x-axis maximum, drawn at the edge. */
    clamped: boolean;
    highlighted: boolean;
  }

  let {
    courses,
    fce,
    highlight = new Set<string>(),
    hrefFor,
    fill = false,
    class: className = "",
  }: {
    courses: readonly Course[];
    fce: Map<string, FceSummary>;
    /** Courses to mark and label, such as the ones in the student's schedule. */
    highlight?: ReadonlySet<string>;
    hrefFor: (course: Course) => string;
    /** Draw on the page instead of a card, as tall as the rest of the window (the Courses page's chart view). */
    fill?: boolean;
    class?: string;
  } = $props();

  const M = { top: 14, right: 14, bottom: 42, left: 42 };
  const HIT_RADIUS = 24;

  /** Deterministic horizontal jitter in [-0.5, 0.5) so courses with equal units don't stack into a line. */
  function jitter(id: string): number {
    let h = 2_166_136_261;
    for (const ch of id) h = Math.imul(h ^ (ch.codePointAt(0) ?? 0), 16_777_619);
    return ((h >>> 0) % 1000) / 1000 - 0.5;
  }

  const niceCeil = (v: number, step: number): number => Math.max(step, Math.ceil(v / step) * step);
  const diamond = (x: number, y: number, r: number): string =>
    `M${x},${y - r}L${x + r},${y}L${x},${y + r}L${x - r},${y}Z`;

  let width = $state(0);
  let hover = $state<Point | null>(null);
  let pinned = $state<Point | null>(null);
  let focusIndex = $state<number | null>(null);

  const data = $derived.by(() => {
    const list: Omit<Point, "x" | "y" | "clamped" | "highlighted">[] = [];
    let variable = 0;
    let missing = 0;
    for (const course of courses) {
      const s = fce.get(course.id);
      if (s?.hours === null || s?.hours === undefined) {
        missing += 1;
        continue;
      }
      if (course.units === null) {
        variable += 1;
        continue;
      }
      const mini = courseIsMini(course, s);
      list.push({ course, summary: s, units: course.units, mini, unitEq: expectedHours(course.units, mini) ?? 0, hours: s.hours });
    }
    return { list, variable, missing };
  });

  let plot = $state<HTMLElement>();
  let viewportHeight = $state(0);
  /** The plot's distance from the top of the document, so a filling chart reaches the bottom of the first screen. */
  let plotTop = $state(0);
  $effect(() => {
    if (!fill || plot === undefined) return;
    const element = plot;
    const measure = (): void => {
      viewportHeight = window.innerHeight;
      plotTop = element.getBoundingClientRect().top + window.scrollY;
    };
    measure();
    window.addEventListener("resize", measure);
    return () => window.removeEventListener("resize", measure);
  });
  const height = $derived(fill && viewportHeight > 0 ? Math.max(380, Math.round(viewportHeight - plotTop - 24)) : 420);
  const innerW = $derived(Math.max(0, width - M.left - M.right));
  const innerH = $derived(height - M.top - M.bottom);
  const maxUnits = $derived(data.list.reduce((m, d) => Math.max(m, d.unitEq), 0));
  const miniCount = $derived(data.list.filter((d) => d.mini).length);
  const xMax = $derived(niceCeil(Math.min(Math.max(maxUnits, 12), 24), 3));
  const yMax = $derived(niceCeil(Math.max(xMax, data.list.reduce((m, d) => Math.max(m, d.hours), 0)), 5));
  const sx = (u: number): number => M.left + (u / xMax) * innerW;
  const sy = (h: number): number => M.top + innerH - (h / yMax) * innerH;
  const jitterWidth = $derived(innerW > 0 ? Math.min(0.7, (24 / innerW) * xMax) : 0);

  const points = $derived.by((): Point[] => {
    if (innerW <= 0) return [];
    return data.list
      .map((d) => {
        const clamped = d.unitEq > xMax;
        // Jitter spreads a column of points, but never past either end of the axis.
        const u = clamped ? xMax : Math.min(xMax, Math.max(0, d.unitEq + jitter(d.course.id) * jitterWidth));
        const point: Point = {
          course: d.course,
          summary: d.summary,
          units: d.units,
          mini: d.mini,
          unitEq: d.unitEq,
          hours: d.hours,
          x: sx(u),
          y: sy(d.hours),
          clamped,
          highlighted: highlight.has(d.course.id),
        };
        return point;
      })
      .toSorted((a, b) => a.x - b.x || a.y - b.y);
  });
  const marked = $derived(points.filter((p) => p.highlighted));

  /**
   * Where each marked course's code goes: right of its point unless that overlaps a label already placed or leaves
   * the plot, then left, above or below. Codes are 11.5px tabular, about 7px a character.
   */
  const labels = $derived.by(() => {
    // Labels keep clear of every marked point as well as of each other.
    const placed: { x0: number; y0: number; x1: number; y1: number }[] = marked.map((m) => ({
      x0: m.x - 6,
      y0: m.y - 6,
      x1: m.x + 6,
      y1: m.y + 6,
    }));
    const out = new Map<string, { x: number; y: number; anchor: "start" | "end" }>();
    const left = M.left;
    const right = M.left + innerW;
    for (const p of [...marked].sort((a, b) => a.y - b.y || a.x - b.x)) {
      const w = p.course.code.length * 7;
      const h = 14;
      const candidates: { x: number; y: number; anchor: "start" | "end" }[] = [
        { x: p.x + 8, y: p.y, anchor: "start" },
        { x: p.x - 8, y: p.y, anchor: "end" },
        { x: p.x + 6, y: p.y - 13, anchor: "start" },
        { x: p.x + 6, y: p.y + 13, anchor: "start" },
        { x: p.x - 6, y: p.y - 13, anchor: "end" },
        { x: p.x - 6, y: p.y + 13, anchor: "end" },
      ];
      const box = (c: { x: number; y: number; anchor: "start" | "end" }) => {
        const x0 = c.anchor === "start" ? c.x : c.x - w;
        return { x0, y0: c.y - h / 2, x1: x0 + w, y1: c.y + h / 2 };
      };
      const fits = (b: { x0: number; y0: number; x1: number; y1: number }) =>
        b.x0 >= left && b.x1 <= right + M.right &&
        placed.every((o) => b.x1 <= o.x0 || b.x0 >= o.x1 || b.y1 <= o.y0 || b.y0 >= o.y1);
      const choice = candidates.find((c) => fits(box(c))) ?? candidates[0];
      placed.push(box(choice));
      out.set(p.course.id, choice);
    }
    return out;
  });
  const activePoint = $derived(pinned ?? hover ?? (focusIndex === null ? null : (points[focusIndex] ?? null)));

  const xTicks = $derived(Array.from({ length: xMax / 3 + 1 }, (_, i) => i * 3));
  const yStep = $derived(yMax > 30 ? 10 : 5);
  const yTicks = $derived(Array.from({ length: Math.floor(yMax / yStep) + 1 }, (_, i) => i * yStep));
  const diagEnd = $derived(Math.min(xMax, yMax));

  function nearest(event: PointerEvent): Point | null {
    const target = event.currentTarget as SVGSVGElement;
    const rect = target.getBoundingClientRect();
    const px = event.clientX - rect.left;
    const py = event.clientY - rect.top;
    let best: Point | null = null;
    let bestD = HIT_RADIUS * HIT_RADIUS;
    for (const p of points) {
      const d = (p.x - px) ** 2 + (p.y - py) ** 2;
      if (d < bestD) {
        bestD = d;
        best = p;
      }
    }
    return best;
  }

  function open(point: Point): void {
    void goto(hrefFor(point.course));
  }

  function onpointermove(event: PointerEvent): void {
    if (event.pointerType !== "touch") hover = nearest(event);
  }

  function onpointerup(event: PointerEvent): void {
    const p = nearest(event);
    // Touch: the first tap shows the details; the tooltip's link opens the course.
    if (event.pointerType === "touch" || event.pointerType === "pen") pinned = p;
    else if (p) open(p);
  }

  function onkeydown(event: KeyboardEvent): void {
    if (points.length === 0) return;
    const i = focusIndex ?? -1;
    if (event.key === "ArrowRight" || event.key === "ArrowDown") focusIndex = Math.min(points.length - 1, i + 1);
    else if (event.key === "ArrowLeft" || event.key === "ArrowUp") focusIndex = Math.max(0, i - 1);
    else if (event.key === "Home") focusIndex = 0;
    else if (event.key === "End") focusIndex = points.length - 1;
    else if (event.key === "Enter" && focusIndex !== null) {
      const p = points[focusIndex];
      if (p) open(p);
    } else if (event.key === "Escape") {
      focusIndex = null;
      pinned = null;
    } else return;
    event.preventDefault();
  }

  const tooltipWidth = $derived(Math.min(260, width - 16));
  const tooltipStyle = $derived.by(() => {
    if (activePoint === null) return "";
    const flip = activePoint.x + 14 + tooltipWidth > width && activePoint.x - 14 - tooltipWidth >= 8;
    const left = flip
      ? activePoint.x - 14 - tooltipWidth
      : Math.max(8, Math.min(activePoint.x + 14, width - tooltipWidth - 8));
    return `width:${tooltipWidth}px;left:${left}px;top:${Math.max(4, activePoint.y - 20)}px`;
  });
</script>

<div
  class="flex flex-col gap-2 {fill ? '' : 'rounded-card border border-pressed bg-surface px-3 pt-3 pb-4 sm:px-4'} {className}"
>
  <div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-label-md text-ink">
    <span class="flex items-center gap-1.5">
      <svg aria-hidden="true" width="10" height="10"><circle cx="5" cy="5" r="4" class="fill-accent" opacity="0.6" /></svg>
      Course
    </span>
    {#if marked.length > 0}
      <span class="flex items-center gap-1.5">
        <svg aria-hidden="true" width="10" height="10"><circle cx="5" cy="5" r="4" class="fill-strong" /></svg>
        In your schedule
      </span>
    {/if}
    {#if miniCount > 0}
      <span class="flex items-center gap-1.5" title="Minis run half a semester, so they're plotted at twice their units: a 6-unit mini sits at 12.">
        <svg aria-hidden="true" width="10" height="10"><path d={diamond(5, 5, 4.5)} class="fill-warn" opacity="0.8" /></svg>
        Mini, at 2 &times; units
      </span>
    {/if}
    <span class="flex items-center gap-1.5">
      <svg aria-hidden="true" width="16" height="10"><line x1="1" y1="9" x2="15" y2="1" class="stroke-muted" stroke-width="1.5" /></svg>
      Hours equal units
    </span>
    <span class="text-muted" title={FCE_HOURS_HELP}>
      {points.length.toLocaleString("en-US")} plotted{data.missing > 0
        ? `, ${data.missing.toLocaleString("en-US")} without FCE hours`
        : ""}{data.variable > 0 ? `, ${data.variable} with variable units` : ""}
    </span>
  </div>

  <div class="relative" bind:this={plot} bind:clientWidth={width} style="height:{height}px">
    {#if width > 0}
      <!-- A chart the keyboard can step through: arrows move between courses, Enter opens one. -->
      <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
      <svg
        {width}
        {height}
        role="application"
        aria-label="Scatter chart of FCE hours per week against units for {points.length} courses; minis are plotted at twice their units because they run half the semester. Use the arrow keys to step through courses and Enter to open one. The list view shows the same data as a table."
        tabindex="0"
        {onpointermove}
        onpointerleave={() => (hover = null)}
        {onpointerup}
        {onkeydown}
        onblur={() => (focusIndex = null)}
        class="block cursor-crosshair touch-manipulation rounded-tag"
      >
        {#each yTicks as t (t)}
          <line x1={M.left} x2={M.left + innerW} y1={sy(t)} y2={sy(t)} class="stroke-line" stroke-width="1" />
          <text x={M.left - 8} y={sy(t)} dy="0.32em" text-anchor="end" class="tabular fill-muted text-[11px]">{t}</text>
        {/each}
        {#each xTicks as t (t)}
          <line x1={sx(t)} x2={sx(t)} y1={M.top} y2={M.top + innerH} class="stroke-line" stroke-width="1" opacity="0.6" />
          <text x={sx(t)} y={M.top + innerH + 16} text-anchor="middle" class="tabular fill-muted text-[11px]"
            >{t === xMax && maxUnits > xMax ? `${t}+` : t}</text
          >
        {/each}
        <text x={M.left + innerW / 2} y={height - 6} text-anchor="middle" class="fill-ink text-[12px]"
          >{miniCount > 0 ? "Units (minis \u00D7 2)" : "Units"}</text
        >
        <text transform="translate(12 {M.top + innerH / 2}) rotate(-90)" text-anchor="middle" class="fill-ink text-[12px]"
          >FCE hours per week</text
        >
        <line x1={sx(0)} y1={sy(0)} x2={sx(diagEnd)} y2={sy(diagEnd)} class="stroke-muted" stroke-width="1.5" />
        <text x={sx(diagEnd) - 4} y={sy(diagEnd) - 6} text-anchor="end" class="fill-muted text-[11px]">Hours = units</text>
        <text x={M.left + 6} y={M.top + 12} class="fill-muted text-[11px]">Heavier than units</text>
        <text x={M.left + innerW - 6} y={M.top + innerH - 8} text-anchor="end" class="fill-muted text-[11px]"
          >Lighter than units</text
        >

        <g>
          {#each points as p (p.course.id)}
            {#if !p.highlighted}
              {#if p.mini}
                <path
                  d={diamond(p.x, p.y, 4.75)}
                  class={p.clamped ? "fill-surface stroke-warn" : "fill-warn stroke-surface"}
                  fill-opacity={p.clamped ? 1 : 0.6}
                  stroke-width={p.clamped ? 1.5 : 1}
                />
              {:else}
                <circle
                  cx={p.x}
                  cy={p.y}
                  r="4"
                  class={p.clamped ? "fill-surface stroke-accent" : "fill-accent stroke-surface"}
                  fill-opacity={p.clamped ? 1 : 0.5}
                  stroke-width={p.clamped ? 1.5 : 1}
                />
              {/if}
            {/if}
          {/each}
        </g>
        <g>
          {#each marked as p (p.course.id)}
            {#if p.mini}
              <path d={diamond(p.x, p.y, 6)} class="fill-strong stroke-surface" stroke-width="2" />
            {:else}
              <circle cx={p.x} cy={p.y} r="5" class="fill-strong stroke-surface" stroke-width="2" />
            {/if}
          {/each}
          {#each marked as p (p.course.id)}
            {@const label = labels.get(p.course.id) ?? { x: p.x + 8, y: p.y, anchor: "start" }}
            <text
              x={label.x}
              y={label.y}
              text-anchor={label.anchor}
              dy="0.32em"
              class="tabular fill-strong stroke-surface text-[11.5px] font-semibold"
              paint-order="stroke"
              stroke-width="3">{p.course.code}</text
            >
          {/each}
        </g>
        {#if activePoint}
          <circle cx={activePoint.x} cy={activePoint.y} r="8" fill="none" class="stroke-strong" stroke-width="2" pointer-events="none" />
        {/if}
      </svg>
    {/if}

    {#if activePoint}
      {@const s = activePoint.summary}
      {@const tone = workloadTone(workloadRatio(s.hours, activePoint.course.units, activePoint.mini))}
      <div
        role={pinned ? "dialog" : "tooltip"}
        aria-label="{activePoint.course.code} details"
        class="absolute z-20 rounded-[8px] border border-line bg-surface px-3 py-2.5 text-body-2 shadow-popover {pinned
          ? ''
          : 'pointer-events-none'}"
        style={tooltipStyle}
      >
        <div class="flex items-baseline gap-2">
          <span class="tabular font-semibold text-strong">{activePoint.course.code}</span>
          <span class="truncate text-ink">{activePoint.course.title}</span>
        </div>
        <dl class="mt-1.5 grid grid-cols-[auto_1fr] gap-x-3 gap-y-0.5">
          <dt class="text-muted">Hours a week</dt>
          <dd class="tabular font-semibold text-ink">{formatHours(s.hours)}</dd>
          <dt class="text-muted">Units</dt>
          <dd class="tabular text-ink">{activePoint.course.units}{activePoint.mini ? " (mini)" : ""}</dd>
          <dt class="text-muted">Units suggest</dt>
          <dd class="tabular text-ink">{activePoint.unitEq} h/wk</dd>
          <dt class="text-muted">Course rating</dt>
          <dd class="tabular text-ink">{formatRating(s.courseRate)}</dd>
        </dl>
        {#if tone}<p class="mt-1 text-label-md text-ink">{WORKLOAD_TONE_LABEL[tone]}</p>{/if}
        {#if activePoint.mini}
          <p class="mt-0.5 text-label-md text-muted">
            A mini runs half the semester, so {expectedHoursText(activePoint.course.units, true)}.
          </p>
        {/if}
        <p class="mt-0.5 text-label-md text-muted">
          {s.terms} recent {s.terms === 1 ? "term" : "terms"}, {s.responses.toLocaleString("en-US")} responses
        </p>
        {#if pinned}
          <div class="mt-2 flex gap-2">
            <a
              href={hrefFor(activePoint.course)}
              class="inline-flex h-10 flex-1 items-center justify-center rounded-full bg-accent px-3 text-label-lg text-on-accent"
              >Open course</a
            >
            <button
              type="button"
              onclick={() => (pinned = null)}
              class="inline-flex h-10 items-center justify-center rounded-full border border-line-strong px-3 text-label-lg text-ink"
              >Close</button
            >
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
