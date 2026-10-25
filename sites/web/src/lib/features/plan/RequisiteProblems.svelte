<script lang="ts">
  // Courses planned before their requisites are met, each jumping to the course. Hidden when there are none.
  import { ChevronDown, TriangleAlert } from "@lucide/svelte";
  import { termPhase } from "$lib/core/plan";
  import { codeOf } from "$lib/core/requisite-format";
  import { termLabel } from "$lib/core/requisites";
  import { planPage } from "./context";

  const page = planPage();
  const listId = $props.id();
  let open = $state(false);
  const problems = $derived(
    [...page.derived.checks.values()].filter(
      (c) => c.check.level === "problem" && termPhase(c.term, page.derived.current) !== "past",
    ),
  );
</script>

{#if problems.length > 0}
  <section aria-labelledby="{listId}-title" class="rounded-card border border-danger/35 bg-surface">
    <h2 id="{listId}-title">
      <button
        type="button"
        onclick={() => (open = !open)}
        aria-expanded={open}
        aria-controls={listId}
        class="flex min-h-11 w-full items-center gap-2 px-4 text-left"
      >
        <TriangleAlert aria-hidden="true" class="size-4 shrink-0 text-danger" />
        <span class="flex-1 text-label-lg text-strong">
          {problems.length} planned course{problems.length === 1 ? "" : "s"} with unmet requisites
        </span>
        <ChevronDown aria-hidden="true" class="size-4 text-muted transition-transform {open ? 'rotate-180' : ''}" />
      </button>
    </h2>
    {#if open}
      <ul id={listId} class="flex flex-col border-t border-line px-2 py-1.5">
        {#each problems as p (p.planCourseId)}
          <li>
            <button
              type="button"
              onclick={() => page.jumpTo(p.planCourseId, p.term)}
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
    {/if}
  </section>
{/if}
