<!--
  What a course preview shows (Courseweave's CoursePreview body): code, units, title and department, the start of the
  description, prerequisites and the student's requisite status, when the semester offers it, and its FCE hours.
  Everything comes from the shared course data, so a preview of a course already on screen loads nothing new.
-->
<script lang="ts">
  import { Check, Clock, Info, TriangleAlert } from "@lucide/svelte";
  import { departmentName } from "$lib/core/departments";
  import { catalogTitle } from "$lib/core/explorer/unoffered";
  import { meetingSummary } from "$lib/core/explorer/summary";
  import {
    courseIsMini,
    expectedHours,
    formatHours,
    formatRating,
    workloadRatio,
    workloadTone,
  } from "$lib/core/fce";
  import { LEVEL_STYLE, summarizeCheck } from "$lib/core/record/check-summary";
  import { codeOf, formatRequisite } from "$lib/core/requisite-format";
  import { checkCourse, termLabel } from "$lib/core/requisites";
  import { topSections } from "$lib/core/sections";
  import { catalog, resource } from "$lib/data";
  import { device, resolveTerm } from "$lib/state";
  import { catalogIndex, courseDetails, fceSummary, termData } from "./data.svelte";
  import { termRequisites } from "./requisite-context.svelte";

  let {
    courseId,
    termId,
    id,
    room = false,
  }: {
    /** Five-digit course id. */
    courseId: string;
    /** The semester to describe it in; null uses the last one visited. */
    termId: string | null;
    /** DOM id of the text, for aria-describedby. */
    id?: string;
    /** Leaves room beside the heading for a close button in the card's corner. */
    room?: boolean;
  } = $props();

  const terms = resource(() => catalog.terms());
  const semester = $derived(termId ?? resolveTerm(null, terms.value ?? [], device.lastTerm));
  const term = $derived(semester === null ? null : termData(semester));
  const index = $derived(catalogIndex());
  const details = $derived(courseDetails(courseId));
  const fce = $derived(fceSummary());

  const course = $derived(term?.value?.courseById.get(courseId) ?? null);
  const entry = $derived(index.value?.courses[courseId]);
  const code = $derived(course?.code ?? codeOf(courseId));
  const title = $derived(course?.title ?? catalogTitle(entry?.title));
  const units = $derived.by((): string => {
    if (course) return course.units === null ? "Variable units" : `${course.units} units`;
    if (entry?.unitsMax !== undefined) return `${entry.units ?? 0}\u2013${entry.unitsMax} units`;
    return entry?.units === null || entry?.units === undefined ? "Variable units" : `${entry.units} units`;
  });
  const passFail = $derived(course?.passFail ?? entry?.passFail ?? false);
  const description = $derived(details.value?.description);
  const summary = $derived(fce.value?.courses.get(courseId) ?? null);
  const unitCount = $derived(course ? course.units : (entry?.units ?? null));
  const mini = $derived(courseIsMini(course, summary));
  const expected = $derived(expectedHours(unitCount, mini));
  const tone = $derived(
    summary?.hours === null || summary?.hours === undefined
      ? null
      : workloadTone(workloadRatio(summary.hours, unitCount, mini)),
  );

  const requisites = $derived(semester === null ? null : termRequisites(semester));
  const ctx = $derived(requisites?.ctx ?? null);
  // Nothing taken, planned or scheduled: every check would read "missing", which says nothing.
  const check = $derived(ctx !== null && ctx.credits.size > 0 && entry ? checkCourse(courseId, entry, ctx) : null);
  const status = $derived(check && entry ? summarizeCheck(check, entry) : null);
  const icons = { check: Check, clock: Clock, info: Info, "triangle-alert": TriangleAlert };
  const StatusIcon = $derived(status ? icons[LEVEL_STYLE[status.level].icon] : null);

  const meets = $derived(course ? meetingSummary(course) : null);
  const tops = $derived(course ? topSections(course).length : 0);
  const termName = $derived(term?.value?.meta.name ?? (semester === null ? null : termLabel(semester)));
  const loading = $derived(!title && (index.status === "loading" || term?.status === "loading"));
</script>

<div {id} class="flex flex-col gap-2.5 text-body-2 text-ink">
  {#if loading}
    <p class="text-muted">Loading {code}&hellip;</p>
  {:else}
    <div class="min-w-0 {room ? 'pr-10' : ''}">
      <div class="flex flex-wrap items-baseline gap-x-2">
        <span class="tabular text-heading-5 text-strong">{code}</span>
        <span class="text-label-md text-muted">{units}{passFail ? ", pass/fail" : ""}</span>
      </div>
      {#if title}
        <p class="font-medium text-strong">{title}</p>
      {:else}
        <p class="text-muted">Not in the course catalog</p>
      {/if}
      {#if course}<p class="text-label-md text-muted">{departmentName(course.dept)}</p>{/if}
    </div>

    {#if description}<p class="line-clamp-3 text-ink">{description}</p>{/if}

    {#if entry?.prereqs}
      <p><span class="text-muted">Prerequisites</span> {formatRequisite(entry.prereqs)}</p>
    {/if}

    {#if status && StatusIcon && status.level !== "none"}
      <p class="flex items-start gap-1.5 {LEVEL_STYLE[status.level].color}">
        <StatusIcon aria-hidden="true" class="mt-0.5 size-4 shrink-0" />
        <span>{status.text}</span>
      </p>
    {/if}

    <div class="flex flex-wrap gap-x-4 gap-y-1 border-t border-line pt-2 text-label-md">
      {#if termName !== null}
        <span class={course ? "text-ink" : "text-muted"}>
          {#if course}
            {termName}: {tops > 1 ? meets?.primary : (meets?.primary ?? "offered")}
          {:else if term?.status === "ready"}
            Not offered in {termName}
          {/if}
        </span>
      {/if}
      {#if summary?.hours !== null && summary?.hours !== undefined}
        <span
          class="text-ink"
          title={expected === null ? undefined : `Units suggest about ${formatHours(expected)} hours a week`}
        >
          <span class={tone === "heavy" || tone === "very-heavy" ? "text-warn" : tone === "light" ? "text-ok" : ""}
            >{formatHours(summary.hours)} hrs/wk</span
          >{summary.courseRate === null ? "" : `, rated ${formatRating(summary.courseRate)}`}
        </span>
      {:else if fce.status === "signin"}
        <span class="text-muted">Sign in to see FCEs</span>
      {:else if fce.status === "forbidden"}
        <span class="text-muted">FCEs need a CMU account</span>
      {/if}
    </div>
  {/if}
</div>
