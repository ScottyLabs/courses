<script lang="ts">
  // A planned course in a semester card: drag handle, code, title, pass/fail, offering warning, requisite state,
  // FCE hours, units and options. Courseweave's PlanCourseRow in features/degree/CourseRows.tsx.
  import { GripVertical, TriangleAlert } from "@lucide/svelte";
  import { DRAG_TYPE } from "$lib/core/degree/hooks";
  import { FCE_HOURS_HELP, formatHours } from "$lib/core/fce";
  import type { PlanCheck, PlanCourse } from "$lib/core/plan";
  import { codeOf } from "$lib/core/requisite-format";
  import { planPage } from "./context";
  import CourseOptions from "./CourseOptions.svelte";
  import { courseRowInfo, requisiteDetails } from "./rows";
  import StatusButton from "./StatusButton.svelte";

  let {
    course,
    term,
    scheduled,
    moveTargets,
    removeLabel,
    dropMark,
    onmove,
    onmoveup,
    onmovedown,
    onremove,
    onupdate,
  }: {
    course: PlanCourse;
    term: string;
    scheduled: Set<string> | undefined;
    moveTargets: string[];
    removeLabel?: string;
    /** While a course is dragged over the semester: it would land just before (or after the last) row. */
    dropMark?: "before" | "after";
    onmove: (term: string) => void;
    onmoveup?: () => void;
    onmovedown?: () => void;
    onremove: () => void;
    onupdate: (patch: { passFail?: boolean; units?: number }) => void;
  } = $props();

  const page = planPage();

  const check = $derived<PlanCheck | undefined>(page.derived.checks.get(course.id));
  const info = $derived(
    courseRowInfo({
      course,
      term,
      current: page.derived.current,
      catalog: page.catalog,
      unitsOf: page.derived.unitsOf,
      check,
      scheduled,
    }),
  );
  const title = $derived(page.lookup(course.courseId).title);
  const hours = $derived(page.fceHours(course.courseId));
  const code = $derived(codeOf(course.courseId));
  const href = $derived(page.href(course.courseId, term));

  function ondragstart(event: DragEvent): void {
    event.dataTransfer?.setData(DRAG_TYPE, course.id);
    event.dataTransfer?.setData("text/plain", code);
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
    page.startDrag({ kind: "move", planCourseId: course.id });
  }
</script>

<li
  id="plan-course-{course.id}"
  tabindex="-1"
  data-plan-row={course.id}
  draggable={page.dragEnabled ? "true" : undefined}
  ondragstart={page.dragEnabled ? ondragstart : undefined}
  ondragend={page.dragEnabled ? () => page.endDrag() : undefined}
  class="group relative rounded-tag px-1 outline-none hover:bg-paper focus-visible:ring-2 focus-visible:ring-accent {page.dragEnabled
    ? 'cursor-grab active:cursor-grabbing'
    : ''}"
>
  {#if dropMark}
    <span
      aria-hidden="true"
      class="pointer-events-none absolute inset-x-1 h-0.5 rounded-full bg-accent {dropMark === 'before'
        ? '-top-px'
        : '-bottom-px'}"
    ></span>
  {/if}
  <div class="flex min-h-11 items-center gap-1.5 md:min-h-9">
    {#if page.dragEnabled}
      <GripVertical aria-hidden="true" class="size-3.5 shrink-0 text-faint group-hover:text-muted" />
    {/if}
    <!-- In a draggable row the link mustn't start its own drag: the row's drag moves the course. -->
    <a
      {href}
      draggable={page.dragEnabled ? "false" : undefined}
      class="tabular shrink-0 text-label-lg text-strong hover:text-accent-ink hover:underline"
    >
      {code}
    </a>
    <span class="min-w-0 flex-1 truncate text-body-2 text-ink" {title}>{title ?? "Not in the catalog"}</span>
    {#if info.passFail}
      <span
        title={info.passFailOnly ? "Pass/fail only: units are not factorable" : "Taking pass/fail: units are not factorable"}
        class="shrink-0 rounded-tag border border-pressed px-1 text-label-sm font-semibold text-muted"
      >
        P/F
      </span>
    {/if}
    {#if info.offeringWarning}
      <span title={info.offeringWarning} class="shrink-0 text-warn">
        <TriangleAlert aria-hidden="true" class="size-3.5" />
        <span class="sr-only">{info.offeringWarning}</span>
      </span>
    {/if}
    {#if info.level !== null && info.statusText !== null}
      <StatusButton
        level={info.level}
        text={requisiteDetails(check, info.statusText)}
        {code}
      />
    {/if}
    <span class="tabular hidden w-10 shrink-0 text-right text-label-md font-normal text-muted sm:inline" title={FCE_HOURS_HELP}>
      {hours === null ? "" : `${formatHours(hours)}h`}
    </span>
    <span class="tabular w-9 shrink-0 text-right text-label-md text-ink" title={info.variable ? "Variable units" : undefined}>
      {info.units}{info.variable ? "*" : ""}u
    </span>
    <CourseOptions
      {course}
      {info}
      {href}
      {moveTargets}
      {removeLabel}
      {onmove}
      {onmoveup}
      {onmovedown}
      {onremove}
      {onupdate}
      {check}
    />
  </div>
  </li>
