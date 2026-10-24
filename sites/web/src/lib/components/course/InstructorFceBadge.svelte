<script lang="ts">
  import { FCE_HOURS_HELP, formatHours, instructorCourseStats, termName } from "$lib/core/fce";
  import { courseFce } from "./data.svelte";

  let {
    courseId,
    instructor,
    class: className = "",
  }: {
    courseId: string;
    /** As the schedule names them, such as "Iliano Cervesato". */
    instructor: string;
    class?: string;
  } = $props();

  // Member data: nothing shows while signed out or when they haven't taught the course before.
  const load = $derived(courseFce(courseId));
  const rows = $derived(load.value);
  const stats = $derived(rows ? instructorCourseStats(rows, instructor) : null);
  const title = $derived(
    stats
      ? `${instructor} in this course: teaching ${stats.teachingRate?.toFixed(2) ?? "n/a"} of 5, course ${stats.courseRate?.toFixed(2) ?? "n/a"} of 5, ${formatHours(stats.hours)} hours a week. ${stats.terms} ${stats.terms === 1 ? "semester" : "semesters"} taught, last in ${termName(stats.last)}. ${FCE_HOURS_HELP}`
      : "",
  );
</script>

{#if stats}
  <span {title} class="inline-flex items-baseline gap-2 text-label-md text-muted {className}">
    {#if stats.teachingRate !== null}
      <span><span class="tabular font-semibold text-ink">{stats.teachingRate.toFixed(1)}</span> teaching</span>
    {/if}
    {#if stats.hours !== null}
      <span><span class="tabular font-semibold text-ink">{formatHours(stats.hours)}</span> hrs/wk</span>
    {/if}
    <span class="sr-only">{title}</span>
  </span>
{/if}
