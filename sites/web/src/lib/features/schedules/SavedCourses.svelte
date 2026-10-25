<!--
  Saved courses offered this semester and not in the schedule yet, one tap to add (Courseweave's SavedCourses).
  Hovering or focusing one previews its sections on the calendar.
-->
<script lang="ts">
  import { Plus } from "@lucide/svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import type { Term } from "$lib/core/data";
  import { coursePath } from "$lib/core/routes";
  import type { ScheduleView } from "$lib/core/state/schedule-view";
  import type { Course } from "$lib/core/types";
  import { saved } from "$lib/state";

  let {
    term,
    view,
    onadd,
    onpreview,
  }: {
    term: Term;
    view: ScheduleView;
    onadd: (course: Course) => void;
    onpreview: (courseId: string | null) => void;
  } = $props();

  const courses = $derived(
    saved.ids
      .map((id) => term.courseById.get(id))
      .filter((c): c is Course => c !== undefined && !view.hasCourse(c.id)),
  );
  const offSemester = $derived(saved.ids.filter((id) => !term.courseById.has(id)).length);
</script>

{#if courses.length === 0}
  <p class="text-body-2 text-muted">
    {saved.ids.length === 0
      ? "Courses you save on the course list show up here, ready to add."
      : offSemester === saved.ids.length
        ? `None of your saved courses are offered in ${term.meta.name}.`
        : "Every saved course offered this semester is in this schedule."}
  </p>
{:else}
  <ul class="-mx-2 flex flex-col" aria-label="Saved courses offered in {term.meta.name}">
    {#each courses as course (course.id)}
      <li
        class="flex items-center gap-2.5 rounded-tag py-0.5 pl-2 hover:bg-hover"
        onmouseenter={() => onpreview(course.id)}
        onmouseleave={() => onpreview(null)}
      >
        <a href={coursePath(course.code, term.meta.id)} class="tabular code-column text-label-lg text-strong hover:underline"
          >{course.code}</a
        >
        <span class="min-w-0 flex-1 truncate text-body-2 text-ink" title={course.title}>{course.title}</span>
        <IconButton
          label="Add {course.code} to the schedule"
          icon={Plus}
          size="md"
          class="lg:size-8"
          onfocus={() => onpreview(course.id)}
          onblur={() => onpreview(null)}
          onclick={() => {
            onadd(course);
            onpreview(null);
          }}
        />
      </li>
    {/each}
  </ul>
{/if}
