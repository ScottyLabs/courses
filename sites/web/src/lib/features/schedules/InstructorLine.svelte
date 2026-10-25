<!--
  A section's instructors, each with their recent FCEs for the course (the shared InstructorFceBadge): teaching rating
  and reported hours. Each name previews the instructor (decision 41), since options can't hold links. Member data:
  signed out, term documents have no instructors, so nothing shows.
-->
<script lang="ts">
  import InstructorFceBadge from "$lib/components/course/InstructorFceBadge.svelte";
  import { instructorPreview } from "$lib/components/instructor/preview";

  let {
    courseId,
    instructors,
    termId,
  }: {
    courseId: string;
    instructors: readonly string[];
    /** The semester the sections are in, for the names' previews. */
    termId?: string | null;
  } = $props();
</script>

<span class="flex flex-wrap gap-x-3 gap-y-0.5 text-label-md font-normal">
  {#each instructors as name (name)}
    <span class="inline-flex flex-wrap items-baseline gap-x-1.5">
      <span {...instructorPreview(name, termId)}>{name}</span>
      <InstructorFceBadge {courseId} instructor={name} />
    </span>
  {/each}
</span>
