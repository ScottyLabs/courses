<!--
  A section's instructors, each with their recent FCEs for the course (Courseweave's InstructorFceBadge): teaching
  rating and reported hours. Member data: signed out, term documents have no instructors, so nothing shows.
-->
<script lang="ts">
  import { catalog, resource } from "$lib/data";
  import { FCE_HOURS_HELP, formatHours, instructorCourseStats, termName } from "$lib/core/fce";

  let { courseId, instructors }: { courseId: string; instructors: readonly string[] } = $props();

  const rows = resource(() => catalog.courseFce(courseId));
</script>

<span class="flex flex-wrap gap-x-3 gap-y-0.5 text-label-md font-normal">
  {#each instructors as name (name)}
    {@const stats = rows.value === undefined ? null : instructorCourseStats(rows.value, name)}
    <span class="inline-flex flex-wrap items-baseline gap-x-1.5">
      <span>{name}</span>
      {#if stats}
        {@const title = `${name} in this course: teaching ${stats.teachingRate?.toFixed(2) ?? "n/a"} of 5, ${formatHours(stats.hours)} hours a week, last taught in ${termName(stats.last)}. ${FCE_HOURS_HELP}`}
        <span {title} class="tabular opacity-90">
          {#if stats.teachingRate !== null}<strong class="font-semibold">{stats.teachingRate.toFixed(1)}</strong>
            teaching{/if}{#if stats.teachingRate !== null && stats.hours !== null}&nbsp;&middot;&nbsp;{/if}{#if stats.hours !== null}<strong
              class="font-semibold">{formatHours(stats.hours)}</strong
            >&nbsp;h/wk{/if}
          <span class="sr-only">. {title}</span>
        </span>
      {/if}
    </span>
  {/each}
</span>
