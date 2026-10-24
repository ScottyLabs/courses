<!--
  A section's instructors, each with their recent FCEs for the course (the shared InstructorFceBadge): teaching rating
  and reported hours. Member data: signed out, term documents have no instructors, so nothing shows.
-->
<script lang="ts">
  import InstructorFceBadge from "$lib/components/course/InstructorFceBadge.svelte";

  let { courseId, instructors }: { courseId: string; instructors: readonly string[] } = $props();
</script>

<span class="flex flex-wrap gap-x-3 gap-y-0.5 text-label-md font-normal">
  {#each instructors as name (name)}
    <span class="inline-flex flex-wrap items-baseline gap-x-1.5">
      <span>{name}</span>
      <InstructorFceBadge {courseId} instructor={name} />
    </span>
  {/each}
</span>
