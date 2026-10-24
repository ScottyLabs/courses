<script lang="ts">
  // A planned course as a spreadsheet row in the detailed layout: everything the card row has, plus the
  // requirements it satisfies, when it's usually offered and its prerequisites. Courseweave's PlanCourseTableRow.
  import { GripVertical, TriangleAlert } from "@lucide/svelte";
  import type { StatusFn } from "$lib/core/catalog/chip-context";
  import { DRAG_TYPE } from "$lib/core/degree/hooks";
  import { FCE_HOURS_HELP, formatHours } from "$lib/core/fce";
  import type { PlanCheck, PlanCourse } from "$lib/core/plan";
  import { seasonsLabel, type Satisfies } from "$lib/core/plan-table";
  import { codeOf } from "$lib/core/requisite-format";
  import { planPage } from "./context";
  import CourseIssues from "./CourseIssues.svelte";
  import CourseOptions from "./CourseOptions.svelte";
  import RequisiteText from "./RequisiteText.svelte";
  import { courseRowInfo } from "./rows";
  import SatisfiesCell from "./SatisfiesCell.svelte";
  import StatusButton from "./StatusButton.svelte";

  let {
    course,
    term,
    scheduled,
    moveTargets,
    removeLabel,
    dropMark,
    satisfies,
    freeElective,
    chipStatus,
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
    dropMark?: "before" | "after";
    satisfies: Satisfies | null;
    freeElective: boolean;
    /** Done, planned or missing for prerequisite courses, as of this semester. */
    chipStatus: StatusFn | undefined;
    onmove: (term: string) => void;
    onmoveup?: () => void;
    onmovedown?: () => void;
    onremove: () => void;
    onupdate: (patch: { passFail?: boolean; units?: number }) => void;
  } = $props();

  const page = planPage();
  const issuesId = $props.id();
  let showIssues = $state(false);

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
  const entry = $derived(page.catalog?.courses[course.courseId]);
  const title = $derived(page.lookup(course.courseId).title);
  const hours = $derived(page.fceHours(course.courseId));
  const code = $derived(codeOf(course.courseId));
  const href = $derived(page.href(course.courseId, term));
  const offered = $derived(seasonsLabel(entry));
  const mark = $derived(
    dropMark === "before"
      ? "[&>td]:shadow-[inset_0_2px_0_var(--color-accent)]"
      : dropMark === "after"
        ? "[&>td]:shadow-[inset_0_-2px_0_var(--color-accent)]"
        : "",
  );

  function ondragstart(event: DragEvent): void {
    event.dataTransfer?.setData(DRAG_TYPE, course.id);
    event.dataTransfer?.setData("text/plain", code);
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
    page.startDrag({ kind: "move", planCourseId: course.id });
  }

  function onunits(event: Event & { currentTarget: HTMLInputElement }): void {
    const text = event.currentTarget.value;
    const n = Number(text);
    onupdate({ units: text !== "" && Number.isFinite(n) ? n : undefined });
  }
</script>

<tr
  id="plan-course-{course.id}"
  tabindex="-1"
  data-plan-row={course.id}
  draggable={page.dragEnabled ? "true" : undefined}
  ondragstart={page.dragEnabled ? ondragstart : undefined}
  ondragend={page.dragEnabled ? () => page.endDrag() : undefined}
  class="group align-top outline-none hover:bg-paper focus-visible:bg-paper {mark} {page.dragEnabled
    ? 'cursor-grab active:cursor-grabbing'
    : ''}"
>
  <td class="sticky left-0 z-[1] bg-surface py-1.5 pr-2 pl-2 whitespace-nowrap group-hover:bg-paper">
    <span class="flex min-h-7 items-center gap-1">
      {#if page.dragEnabled}
        <GripVertical aria-hidden="true" class="size-3.5 shrink-0 text-faint group-hover:text-muted" />
      {:else}
        <span aria-hidden="true" class="w-3.5 shrink-0"></span>
      {/if}
      <a
        {href}
        draggable={page.dragEnabled ? "false" : undefined}
        class="tabular text-label-lg text-strong hover:text-accent-ink hover:underline">{code}</a
      >
    </span>
  </td>
  <td class="max-w-0 py-1.5 pr-2">
    <span class="flex min-h-7 min-w-0 items-center gap-1.5">
      <span class="truncate text-ink" {title}>{title ?? "Not in the catalog"}</span>
      {#if info.passFail}
        <span
          title={info.passFailOnly ? "Pass/fail only: units are not factorable" : "Taking pass/fail: units are not factorable"}
          class="shrink-0 rounded-tag border border-pressed px-1 text-label-sm font-semibold text-muted">P/F</span
        >
      {/if}
    </span>
  </td>
  <td class="tabular py-1.5 pr-2 text-right">
    {#if info.variable}
      <input
        type="number"
        min="0"
        max="48"
        step="1"
        aria-label="Units for {code}"
        value={course.units ?? info.units}
        oninput={onunits}
        class="tabular h-8 w-14 rounded-tag border border-line bg-surface px-1.5 text-right text-body-2 text-ink md:h-7"
      />
    {:else}
      <span class="inline-flex min-h-7 items-center">{info.units}</span>
    {/if}
  </td>
  <td class="py-1.5 pr-2 text-label-md font-normal">
    <span class="flex min-h-7 items-center"><SatisfiesCell {satisfies} {freeElective} /></span>
  </td>
  <td
    class="py-1.5 pr-2 text-label-md font-normal {info.offeringWarning ? 'text-warn' : 'text-ink'}"
    title={info.offeringWarning ?? undefined}
  >
    <span class="flex min-h-7 items-center gap-1">
      {#if info.offeringWarning}
        <TriangleAlert aria-hidden="true" class="size-3.5 shrink-0" />
        <span class="sr-only">{info.offeringWarning}.</span>
      {/if}
      {offered ?? "\u2013"}
    </span>
  </td>
  <td class="py-0.5 pr-2 text-label-md font-normal text-ink">
    <span class="flex min-w-0 items-start gap-1">
      {#if info.level !== null && info.statusText !== null}
        <StatusButton
          level={info.level}
          text={info.statusText}
          {code}
          open={showIssues}
          controls={issuesId}
          ontoggle={() => (showIssues = !showIssues)}
        />
      {/if}
      {#if entry?.prereqs}
        <RequisiteText
          req={entry.prereqs}
          status={chipStatus}
          href={(id) => page.href(id, term)}
          title={(id) => page.lookup(id).title}
          class="min-w-0 pt-[11px] md:pt-[5px]"
        />
      {:else}
        <span class="pt-[11px] leading-[18px] text-muted md:pt-[5px]">{entry ? "None" : "Not in the catalog"}</span>
      {/if}
    </span>
  </td>
  <td class="tabular py-1.5 pr-2 text-right text-label-md font-normal text-muted" title={FCE_HOURS_HELP}>
    <span class="inline-flex min-h-7 items-center">{hours === null ? "" : `${formatHours(hours)}h`}</span>
  </td>
  <td class="py-0.5 pr-1 text-right">
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
    />
  </td>
</tr>
{#if showIssues && check}
  <tr>
    <td colspan="8" class="px-2 pb-1"><CourseIssues id={issuesId} {check} fallback={info.statusText} /></td>
  </tr>
{/if}
