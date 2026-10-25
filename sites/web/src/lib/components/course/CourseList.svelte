<script lang="ts">
  import type { Snippet } from "svelte";
  import { FCE_HOURS_HELP } from "$lib/core/fce";
  import { fceSummary } from "./data.svelte";

  let {
    label,
    allSemesters = false,
    children,
    class: className = "",
  }: {
    /** Accessible name for the list, such as "Courses in Spring 2027". */
    label: string;
    /** Rows show the semesters offering each course instead of its meetings. */
    allSemesters?: boolean;
    /** CourseRow items. */
    children: Snippet;
    class?: string;
  } = $props();

  const fce = $derived(fceSummary());
  const showFce = $derived(fce.status === "ready" && fce.value !== null);
</script>

<!-- The compact list (Courseweave's rows): a header and CourseRows, with columns that appear as it widens. -->
<div class="course-list overflow-hidden rounded-card border border-pressed bg-surface {className}">
  <div
    aria-hidden="true"
    class="course-list-head grid gap-x-3 border-b border-line bg-surface-2 px-3 py-2 text-label-md text-muted sm:px-4"
  >
    <span>Course</span>
    <span>Title</span>
    <span class="col-meets text-right">{allSemesters ? "Offered" : "Meets"}</span>
    <span class="col-hours text-right" title={showFce ? FCE_HOURS_HELP : undefined}>{showFce ? "FCE hrs" : ""}</span>
    <span class="col-rating text-right">{showFce ? "Rating" : ""}</span>
    <span class="text-right">Units</span>
    <span></span>
  </div>
  <div role="list" aria-label={label}>
    {@render children()}
  </div>
</div>

<style>
  .course-list {
    container: course-list / inline-size;
  }

  .course-list-head {
    grid-template-columns: 4.25rem minmax(0, 1fr) 2.75rem 2rem;
  }

  .col-meets,
  .col-hours,
  .col-rating {
    display: none;
  }

  @container course-list (min-width: 34rem) {
    .course-list-head {
      grid-template-columns: 4.25rem minmax(0, 1fr) 4.5rem 2.75rem 2rem;
    }

    .col-hours {
      display: block;
    }
  }

  @container course-list (min-width: 48rem) {
    .course-list-head {
      grid-template-columns: 4.25rem minmax(0, 1fr) 10rem 4.5rem 3rem 2.75rem 2rem;
    }

    .col-meets,
    .col-rating {
      display: block;
    }
  }
</style>
