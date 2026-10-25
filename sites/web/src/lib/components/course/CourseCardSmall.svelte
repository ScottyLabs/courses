<script lang="ts">
  // The Figma's small course card, for saved courses and the record's courses taken and in progress: code and title
  // (the whole card links to the course), department, friends taking it, then units and FCE hours. `children` adds
  // lines above the units (meeting times), `footer` controls under them (a grade).
  import { Clock, Coins, SquareArrowOutUpRight } from "@lucide/svelte";
  import { departmentName } from "$lib/core/departments";
  import { codeOf } from "$lib/core/requisite-format";
  import { coursePath } from "$lib/core/routes";
  import type { Snippet } from "svelte";
  import { catalogIndex, fceSummary, offerings } from "./data.svelte";
  import FriendsTaking from "./FriendsTaking.svelte";

  let {
    courseId,
    termId = null,
    title: titleProp,
    units: unitsProp,
    dept: deptProp,
    deptLabel,
    actions,
    children,
    footer,
    class: className = "",
  }: {
    courseId: string;
    /** The semester to link to and to show friends for; null links to the course without one. */
    termId?: string | null;
    /** Defaults to the catalog's title. */
    title?: string;
    /** Defaults to the catalog's units; null is variable. */
    units?: number | null;
    /** Department code; defaults to the one the offerings document lists. */
    dept?: string;
    /** The department line as written, overriding `dept`. */
    deptLabel?: string;
    /** Extra controls in the top-right corner, such as a remove button. */
    actions?: Snippet;
    /** Lines between the department and the units. */
    children?: Snippet;
    /** Controls at the bottom, above the card's link. */
    footer?: Snippet;
    class?: string;
  } = $props();

  const code = $derived(codeOf(courseId));
  const entry = $derived(catalogIndex().value?.courses[courseId]);
  const title = $derived(titleProp ?? entry?.title ?? offerings().value?.byId.get(courseId)?.title ?? "");
  const dept = $derived(deptProp ?? offerings().value?.byId.get(courseId)?.dept ?? "");
  const units = $derived.by(() => {
    const value = unitsProp === undefined ? (entry?.unitsMax === undefined ? (entry?.units ?? null) : null) : unitsProp;
    if (value !== null) return `${value} ${value === 1 ? "unit" : "units"}`;
    if (entry?.unitsMax !== undefined) return `${entry.units ?? 0}\u2013${entry.unitsMax} units`;
    return entry === undefined ? "" : "Variable units";
  });
  const hours = $derived(fceSummary().value?.courses.get(courseId)?.hours ?? null);
  const href = $derived(coursePath(code, termId));
</script>

<article class="relative flex min-w-0 flex-col gap-1 rounded-card border border-pressed bg-surface px-5 py-4 text-ink {className}">
  <div class="flex items-start gap-3">
    <h3 class="min-w-0 flex-1 text-heading-6 text-ink">
      <a {href} data-course-preview="off" class="outline-none after:absolute after:inset-0 after:rounded-card after:content-[''] focus-visible:after:animate-focus-ring focus-visible:after:outline-2 focus-visible:after:outline-accent">
        <span class="tabular mr-2 font-semibold text-strong">{code}</span>{title}
      </a>
    </h3>
    <div class="pointer-events-none relative z-10 -mt-1 -mr-2 flex shrink-0 items-center gap-1">
      {#if actions}<span class="pointer-events-auto">{@render actions()}</span>{/if}
      <SquareArrowOutUpRight aria-hidden="true" class="m-2 size-5 text-ink" />
    </div>
  </div>
  <div class="flex items-center justify-between gap-3">
    <p class="min-w-0 truncate text-label-lg font-medium text-muted">
      {deptLabel ?? (dept ? departmentName(dept) : "")}
    </p>
    {#if termId}
      <FriendsTaking {courseId} {termId} size="xs" class="relative z-10 shrink-0" />
    {/if}
  </div>
  {@render children?.()}
  <!-- The units line, or the footer when there is one, sits at the bottom of a card stretched to its row. -->
  <p class="flex flex-wrap items-center gap-x-3 gap-y-1 text-body-2 font-medium {footer ? '' : 'mt-auto'}">
    {#if units}
      <span class="inline-flex items-center gap-1"><Coins aria-hidden="true" class="size-[18px]" />{units}</span>
    {/if}
    {#if hours !== null}
      <span aria-hidden="true" class="h-4 w-px bg-line-strong"></span>
      <span class="inline-flex items-center gap-1"
        ><Clock aria-hidden="true" class="size-[18px]" /><span class="tabular">{hours.toFixed(1)}</span> hrs/week</span
      >
    {/if}
  </p>
  {#if footer}<div class="relative z-10 mt-auto">{@render footer()}</div>{/if}
</article>
