<!--
  The final exams of a schedule's courses: the registrar's exam for each course and section (or the student's SIO
  import for courses the registrar's file lacks), warnings for overlaps, back-to-back exams and three or more in 24
  hours, and a calendar file. Shown in the schedules page's final exams dialog.
-->
<script lang="ts">
  import { CalendarArrowDown, TriangleAlert } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { download } from "$lib/components/ui/download";
  import type { FinalExam } from "$lib/api/types";
  import type { RegistrarFinal } from "$lib/data";
  import { courseColorStyle } from "$lib/core/colors";
  import { coursePath } from "$lib/core/routes";
  import type { ResolvedEntry } from "$lib/core/state/schedule-view";
  import { formatTimeRange } from "$lib/core/time";
  import { fileName } from "$lib/features/schedules/planner";
  import { examDay, finalWarnings, finalsIcs, hasRoom, matchFinals, type Exam } from "./finals";

  let {
    entries,
    registrar,
    sio = [],
    termId,
    termName,
    scheduleName,
  }: {
    entries: readonly ResolvedEntry[];
    /** The registrar's schedule; null when it isn't published. */
    registrar: readonly RegistrarFinal[] | null;
    sio?: readonly FinalExam[];
    termId: string;
    termName: string;
    scheduleName: string;
  } = $props();

  const mine = $derived(matchFinals(entries, registrar, sio));
  const warnings = $derived(finalWarnings(mine));
  const exams = $derived(mine.map((f) => f.exam).filter((e): e is Exam => e !== null));
  const sorted = $derived(
    mine.toSorted((a, b) => {
      if (a.exam === null || b.exam === null) return (a.exam === null ? 1 : 0) - (b.exam === null ? 1 : 0);
      return a.exam.date.localeCompare(b.exam.date) || a.exam.start - b.exam.start;
    }),
  );

  function ics(): void {
    download(
      new Blob([finalsIcs(exams, `${scheduleName} finals (${termName})`)], { type: "text/calendar;charset=utf-8" }),
      `${fileName(scheduleName)} ${termId} finals.ics`,
    );
  }

  const NOTE = {
    section: "",
    course: "Same exam for every section",
    sio: "From your SIO calendar",
    pick: "Pick a section to see its exam",
    none: "No exam listed",
  } as const;
</script>

<div class="flex flex-col gap-3">
  {#if mine.length === 0}
    <p class="text-body-2 text-muted">Add courses to see their final exams.</p>
  {:else}
    {#if warnings.length > 0}
      <ul class="flex flex-col gap-1.5" aria-label="Exam warnings">
        {#each warnings as w (`${w.kind}-${w.codes.join()}`)}
          <li
            class="flex items-start gap-2 rounded-field px-3 py-2 text-body-2 {w.kind === 'overlap'
              ? 'bg-danger-bg text-danger'
              : 'bg-warn-bg text-ink'}"
          >
            <TriangleAlert aria-hidden="true" class="mt-0.5 size-4 shrink-0 {w.kind === 'overlap' ? '' : 'text-warn'}" />
            {w.message}
          </li>
        {/each}
      </ul>
    {/if}
    <ul class="divide-y divide-line rounded-card border border-line bg-surface">
      {#each sorted as f (f.courseId)}
        <li class="flex flex-wrap items-center gap-x-3 gap-y-0.5 px-3 py-2.5" style={courseColorStyle(f.color)}>
          <span aria-hidden="true" class="course-swatch h-4 w-1 shrink-0 rounded-full"></span>
          <a href={coursePath(f.code, termId)} class="tabular code-column text-label-lg text-strong hover:underline"
            >{f.code}</a
          >
          {#if f.exam}
            <span class="min-w-0 flex-1 basis-48 text-body-2 text-ink">
              {examDay(f.exam.date)}, {formatTimeRange(f.exam.start, f.exam.end)}
              {#if f.exam.mode === "Cancelled"}<span class="text-muted">(cancelled)</span>{/if}
            </span>
            <span class="shrink-0 text-label-md font-normal {hasRoom(f.exam) ? 'text-ink' : 'text-muted'}">
              {f.exam.mode === "Remote" ? "Remote" : (f.exam.location ?? "Room to be announced")}
            </span>
            <span class="w-full pl-[5.75rem] text-label-md font-normal text-muted">
              {f.match === "section" ? `Section ${f.exam.section ?? ""}` : NOTE[f.match]}
            </span>
          {:else}
            <span class="min-w-0 flex-1 basis-48 text-body-2 text-muted">{NOTE[f.match]}</span>
          {/if}
        </li>
      {/each}
    </ul>
    <div class="flex flex-wrap items-center gap-3">
      <Button size="sm" icon={CalendarArrowDown} disabled={exams.length === 0} onclick={ics}>Download exams (.ics)</Button>
    </div>
    <p class="text-label-md font-normal text-muted">
      Check exam times with your instructors and SIO: the registrar's schedule can change, and some courses hold their
      own exams.
    </p>
  {/if}
</div>
