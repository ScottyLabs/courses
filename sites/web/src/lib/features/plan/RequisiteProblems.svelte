<script lang="ts">
  // The rail's "Requisites": courses planned before their requisites are met, each jumping to the course.
  import { TriangleAlert } from "@lucide/svelte";
  import type { PlanCheck } from "$lib/core/plan";
  import { codeOf } from "$lib/core/requisite-format";
  import { termLabel } from "$lib/core/requisites";

  let {
    problems,
    onjump,
  }: {
    /** The plan's checks at the "problem" level, in semesters that haven't passed. */
    problems: PlanCheck[];
    onjump: (planCourseId: string, term: string) => void;
  } = $props();
</script>

<div class="flex flex-col gap-2">
  <p class="flex items-start gap-2 text-body-2 text-ink">
    <TriangleAlert aria-hidden="true" class="mt-0.5 size-4 shrink-0 text-danger" />
    <span>
      {problems.length === 1 ? "1 planned course comes" : `${problems.length} planned courses come`} before
      {problems.length === 1 ? "its" : "their"} requisites are met.
    </span>
  </p>
  <ul class="-mx-2 flex flex-col">
    {#each problems as p (p.planCourseId)}
      <li>
        <button
          type="button"
          onclick={() => onjump(p.planCourseId, p.term)}
          class="flex min-h-11 w-full items-start gap-3 rounded-field px-2 py-1.5 text-left hover:bg-paper focus-visible:-outline-offset-2"
        >
          <span class="tabular w-14 shrink-0 text-label-lg text-strong">{codeOf(p.courseId)}</span>
          <span class="min-w-0 flex-1 text-body-2">
            <span class="block text-danger">{p.check.issues.find((i) => i.severity === "error")?.message}</span>
            <span class="text-label-md font-normal text-muted">{termLabel(p.term)}</span>
          </span>
        </button>
      </li>
    {/each}
  </ul>
</div>
