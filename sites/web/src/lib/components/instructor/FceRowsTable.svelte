<script lang="ts">
  import type { Snippet } from "svelte";
  import DataTable, { type Column } from "../ui/DataTable.svelte";
  import { coursePath } from "$lib/core/routes";
  import { responseRate, type InstructorRow } from "$lib/features/instructors/fce-aggregate";
  import { NO_VALUE, cellNumber, courseCode, semesterName } from "./format";

  let {
    rows,
    caption,
    title,
    empty = "No evaluations in these semesters",
    class: className = "",
  }: {
    rows: readonly InstructorRow[];
    /** Describes the table for screen readers. */
    caption: string;
    /** Heading inside the well. */
    title?: Snippet;
    empty?: string;
    class?: string;
  } = $props();

  const columns: Column[] = [
    { key: "term", label: "Semester" },
    { key: "course", label: "Course" },
    { key: "section", label: "Section" },
    { key: "hours", label: "Workload" },
    { key: "teaching", label: "Teaching" },
    { key: "courseRate", label: "Course Rate" },
    { key: "responses", label: "Respondents" },
    { key: "rate", label: "Response Rate" },
  ];

  function percent(row: InstructorRow): string {
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
  rowKey={(row, index) => `${row.term}-${row.courseId}-${row.section}-${index}`}
  class={className}
>
  {#snippet cell(row, column)}
    {#if column.key === "term"}
      {semesterName(row.term)}
    {:else if column.key === "course"}
      <a href={coursePath(row.courseId)}>{courseCode(row.courseId)}</a>
    {:else if column.key === "section"}
      {row.section}
    {:else if column.key === "hours"}
      {cellNumber(row.hours)}
    {:else if column.key === "teaching"}
      {cellNumber(row.teachingRate)}
    {:else if column.key === "courseRate"}
      {cellNumber(row.courseRate)}
    {:else if column.key === "responses"}
      {row.responses}
    {:else}
      {percent(row)}
    {/if}
  {/snippet}
</DataTable>
