<script lang="ts">
  // The middle course's requisite verdict above the graph, in the course pages' words (summarizeCheck), announced as
  // it changes. With nothing on record it asks for the courses taken instead, as the course pages do.
  import { Info } from "@lucide/svelte";
  import RequisiteLevelIcon from "$lib/components/course/RequisiteLevelIcon.svelte";
  import { codeOf } from "$lib/core/requisite-format";
  import { profilePath } from "$lib/core/routes";
  import type { GraphSummary } from "./status";

  let {
    courseId,
    summary,
    unchecked,
    loading,
    unlocksNothing,
  }: {
    courseId: string;
    summary: GraphSummary | null;
    unchecked: boolean;
    /** The student's record and the semester are still loading. */
    loading: boolean;
    /** No course lists it as a requisite. */
    unlocksNothing: boolean;
  } = $props();
</script>

<div class="flex items-start gap-2 px-4 pt-3 pb-1" aria-live="polite">
  {#if summary === null}
    <p class="text-body-2 text-muted">{#if loading}Checking requisites&hellip;{/if}</p>
  {:else if unchecked && summary.level !== "none"}
    <Info aria-hidden="true" class="mt-[3px] size-4 shrink-0 text-muted" />
    <p class="min-w-0 text-body-2 text-ink">
      <a href={profilePath("taken")} class="text-accent-ink underline underline-offset-2">Add the courses you've taken</a>
      to check {codeOf(courseId)}'s requisites against them.
    </p>
  {:else}
    <RequisiteLevelIcon level={summary.level} class="mt-[3px] size-4" />
    <p class="min-w-0 text-body-2 {summary.level === 'problem' ? 'text-danger' : 'text-ink'}">
      <span class="sr-only">{codeOf(courseId)}: </span>{summary.text}{#if unlocksNothing}<span class="text-muted"
          >. No course lists it as a requisite.</span
        >{/if}
    </p>
  {/if}
</div>
