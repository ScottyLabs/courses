<!--
  What a student does with a course, grouped in one card beside the course page's title and under the detail panel's:
  the semester (on the full page), the schedule action (add it, or where it is with Remove and Open), the requisite
  check with its details, and friends taking it. Rows share the card's left edge and a 12px gap.
-->
<script lang="ts">
  import { ArrowRight } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import FriendsTaking from "$lib/components/course/FriendsTaking.svelte";
  import { getCourseHref } from "$lib/components/course/links";
  import RequisiteStatus from "$lib/components/course/RequisiteStatus.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { termLabel } from "$lib/core/requisites";
  import type { CourseSchedule } from "./course-schedule.svelte";
  import ScheduleActions from "./ScheduleActions.svelte";

  let {
    sched,
    courseId,
    checkTermId,
    nextTerm,
    select,
    class: className = "",
  }: {
    sched: CourseSchedule;
    courseId: string;
    /** The semester requisites are checked for; null while none is known. */
    checkTermId: string | null;
    /** Without sections here (the all-semesters view): the nearest semester that has them. */
    nextTerm?: string;
    /** The semester select, first in the card. */
    select?: Snippet;
    class?: string;
  } = $props();

  const courseHref = getCourseHref();
</script>

<div class="flex min-w-0 flex-col gap-3 rounded-card border border-pressed bg-surface p-4 {className}">
  {@render select?.()}
  {#if sched.course !== null && sched.termId !== null}
    <ScheduleActions {sched} />
  {:else if nextTerm}
    <Button variant="primary" full trailingIcon={ArrowRight} href={courseHref(courseId, nextTerm)}>
      Sections in {termLabel(nextTerm)}
    </Button>
  {/if}
  {#if checkTermId !== null}
    <RequisiteStatus {courseId} termId={checkTermId} />
  {/if}
  {#if sched.termId !== null}
    <FriendsTaking {courseId} termId={sched.termId} class="-ml-1" />
  {/if}
</div>
