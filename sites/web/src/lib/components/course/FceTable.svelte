<script lang="ts">
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import { instructorSlug } from "$lib/core/data";
  import { termName, type FceRow } from "$lib/core/fce";
  import { displayFceName } from "$lib/core/fce-names";
  import { instructorPath } from "$lib/core/routes";

  let {
    rows,
    caption = "Course evaluations by section",
    showCourse = false,
    courseLabel,
    class: className = "",
  }: {
    /** FCE rows, in the order to show them. */
    rows: readonly FceRow[];
    caption?: string;
    /** Instructor pages add a course column; pass `courseLabel` to fill it. */
    showCourse?: boolean;
    courseLabel?: (row: FceRow) => string;
    class?: string;
  } = $props();

  const columns = $derived<Column[]>([
    { key: "term", label: "Semester" },
    ...(showCourse ? [{ key: "course", label: "Course" }] : []),
    { key: "section", label: "Section" },
    { key: "instructor", label: "Instructor" },
    { key: "hours", label: "Workload" },
    { key: "teaching", label: "Teaching" },
    { key: "course-rate", label: "Course Rate" },
    { key: "responses", label: "Respondents" },
    { key: "rate", label: "Resp. Rate" },
  ]);

  function value(row: FceRow, key: string): string {
    switch (key) {
      case "term":
        return termName(row.term);
      case "section":
        return row.section;
      case "hours":
        return row.responses > 0 && row.hours !== null ? row.hours.toFixed(2) : "-";
      case "teaching":
        return row.responses > 0 && row.teachingRate !== null ? row.teachingRate.toFixed(2) : "-";
      case "course-rate":
        return row.responses > 0 && row.courseRate !== null ? row.courseRate.toFixed(2) : "-";
      case "responses":
        return String(row.responses);
      case "rate":
        return row.enrolled > 0 ? `${((row.responses / row.enrolled) * 100).toFixed(1)}%` : "-";
      default:
        return "";
    }
  }
</script>

<DataTable
  {columns}
  {rows}
  {caption}
  variant="well"
  empty="No evaluations match these options."
  rowKey={(row, index) => `${row.term}-${row.section}-${row.instructor}-${index}`}
  class={className}
>
  {#snippet cell(row, column)}
    {#if column.key === "instructor"}
      {#if row.instructor}
        {@const name = displayFceName(row.instructor)}
        <a
          href={instructorPath(instructorSlug(name))}
          class="underline decoration-line-strong underline-offset-2 hover:text-accent-ink hover:decoration-current"
          >{row.instructor}</a
        >
      {:else}
        -
      {/if}
    {:else if column.key === "course"}
      {courseLabel?.(row) ?? ""}
    {:else if column.key === "section" && row.mini !== null}
      {row.section} <span class="text-label-md text-muted">mini</span>
    {:else}
      {value(row, column.key)}
    {/if}
  {/snippet}
</DataTable>
