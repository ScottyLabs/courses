<script lang="ts">
  // FCE rows as the Figma's table in a gray well: semester, the course (instructor pages), section, the instructor
  // (course pages), workload, ratings, respondents and response rate.
  import type { Snippet } from "svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import { instructorSlug } from "$lib/core/data";
  import { termName, type FceRow } from "$lib/core/fce";
  import { displayFceName } from "$lib/core/fce-names";
  import { codeOf } from "$lib/core/requisite-format";
  import { ALL_TERMS, coursePath, instructorPath } from "$lib/core/routes";
  import { responseRate } from "./fce-aggregate";
  import { NO_VALUE, hoursText, ratingText } from "./fce-format";

  type Row = FceRow & { courseId?: string };

  let {
    rows,
    caption = "Course evaluations by section",
    title,
    empty = "No evaluations match these options.",
    showCourse = false,
    showInstructor = true,
    class: className = "",
  }: {
    /** FCE rows, in the order to show them. */
    rows: readonly Row[];
    /** Describes the table for screen readers. */
    caption?: string;
    /** Heading inside the well. */
    title?: Snippet;
    empty?: string;
    /** A column linking each row's course (instructor pages); rows need `courseId`. */
    showCourse?: boolean;
    /** A column linking each row's instructor (course pages). */
    showInstructor?: boolean;
    class?: string;
  } = $props();

  const columns = $derived<Column[]>([
    { key: "term", label: "Semester" },
    ...(showCourse ? [{ key: "course", label: "Course" }] : []),
    { key: "section", label: "Section" },
    ...(showInstructor ? [{ key: "instructor", label: "Instructor" }] : []),
    { key: "hours", label: "Workload" },
    { key: "teaching", label: "Teaching" },
    { key: "course-rate", label: "Course Rate" },
    { key: "responses", label: "Respondents" },
    { key: "rate", label: "Response Rate" },
  ]);

  /** A figure, or a dash for sections nobody answered: hours to one decimal, ratings to two, as everywhere. */
  function figure(row: Row, value: number | null, format: (value: number) => string): string {
    return row.responses > 0 && value !== null ? format(value) : NO_VALUE;
  }

  function percent(row: Row): string {
    const rate = responseRate(row);
    return rate === null ? NO_VALUE : `${rate.toFixed(1)}%`;
  }
</script>

<DataTable
  {columns}
  {rows}
  {caption}
  {title}
  {empty}
  rowKey={(row, index) => `${row.term}-${row.courseId ?? ""}-${row.section}-${row.instructor}-${index}`}
  class={className}
>
  {#snippet cell(row, column)}
    {#if column.key === "term"}
      {termName(row.term)}
    {:else if column.key === "course"}
      {#if row.courseId}<a href={coursePath(row.courseId)}>{codeOf(row.courseId)}</a>{:else}{NO_VALUE}{/if}
    {:else if column.key === "section"}
      {row.section}{#if row.mini !== null}&nbsp;<span class="text-label-md text-muted">mini</span>{/if}
    {:else if column.key === "instructor"}
      {#if row.instructor}
        <a
          href={instructorPath(instructorSlug(displayFceName(row.instructor)), ALL_TERMS)}
          class="underline decoration-line-strong underline-offset-2 hover:text-accent-ink hover:decoration-current"
          >{displayFceName(row.instructor)}</a
        >
      {:else}
        {NO_VALUE}
      {/if}
    {:else if column.key === "hours"}
      {figure(row, row.hours, hoursText)}
    {:else if column.key === "teaching"}
      {figure(row, row.teachingRate, ratingText)}
    {:else if column.key === "course-rate"}
      {figure(row, row.courseRate, ratingText)}
    {:else if column.key === "responses"}
      {row.responses}
    {:else}
      {percent(row)}
    {/if}
  {/snippet}
</DataTable>
