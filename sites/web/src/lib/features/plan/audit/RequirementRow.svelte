<script lang="ts">
  // One requirement: its state, target, progress, the courses counted toward it, ways to satisfy what's left and
  // the rules not checked here. Groups expand to their children.
  import { ChevronRight } from "@lucide/svelte";
  import { describeTarget } from "$lib/core/audit";
  import { displayState, STATE_LABEL } from "$lib/core/degree/audit/format";
  import type { RequirementResult } from "$lib/core/plan-types";
  import AssignedChip from "./AssignedChip.svelte";
  import CourseLeafRow from "./CourseLeafRow.svelte";
  import Notes from "./Notes.svelte";
  import Progress from "./Progress.svelte";
  import RequirementList from "./RequirementList.svelte";
  import StateIcon from "./StateIcon.svelte";
  import type { TreeContext } from "./tree";
  import WaysToSatisfy from "./WaysToSatisfy.svelte";

  let { r, ctx, depth }: { r: RequirementResult; ctx: TreeContext; depth: number } = $props();

  const childrenId = $props.id();
  const display = $derived(displayState(r));
  const node = $derived(ctx.node(r.nodeId));
  const isGroup = $derived(r.kind === "group" || r.children.length > 0);
  const done = $derived(display === "complete" || display === "complete-with-plan");
  // Unfinished requirements start open (a few levels deep); finished ones start closed.
  // svelte-ignore state_referenced_locally
  let open = $state(!(display === "complete" || display === "complete-with-plan") && display !== "skipped" && depth < 3);
  const target = $derived(node ? describeTarget(node) : null);
  const needsWork = $derived(!done && display !== "manual" && display !== "skipped");
</script>

{#if r.kind === "course" && r.courseId !== undefined}
  <CourseLeafRow {r} {ctx} />
{:else}
  <li id="req-{r.nodeId}" class="border-t border-line first:border-t-0">
    <div class="flex items-start gap-2 py-2">
      {#if isGroup}
        <button
          type="button"
          aria-expanded={open}
          aria-controls={childrenId}
          aria-label="{open ? 'Collapse' : 'Expand'} {r.name}"
          onclick={() => (open = !open)}
          class="-my-1.5 -ml-1.5 grid size-10 shrink-0 place-items-center rounded-full text-muted hover:bg-hover hover:text-ink md:-my-1 md:size-7"
        >
          <ChevronRight aria-hidden="true" class="size-4 transition-transform {open ? 'rotate-90' : ''}" />
        </button>
      {/if}
      <StateIcon state={display} class="mt-0.5 size-4" />
      <div class="min-w-0 flex-1">
        <div class="flex flex-wrap items-baseline gap-x-2 gap-y-0.5">
          <span class="text-label-lg {display === 'skipped' ? 'text-muted' : 'text-strong'}">{r.name}</span>
          <span class="sr-only">{STATE_LABEL[display]}</span>
          {#if target}<span class="text-label-md text-muted">{target}</span>{/if}
        </div>

        {#if r.units}
          <Progress p={r.units} unit="units" name={r.name} />
        {:else if r.courses && r.courses.need > 1}
          <Progress p={r.courses} unit="courses" name={r.name} />
        {/if}

        {#if r.assigned.length > 0}
          <ul class="mt-1.5 flex flex-wrap gap-1.5">
            {#each r.assigned as a, i (`${a.courseId}-${i}`)}
              <li><AssignedChip {a} {ctx} /></li>
            {/each}
          </ul>
        {/if}

        {#if display === "manual"}
          <p class="mt-1 text-label-md text-muted">Your advisor or the registrar confirms this one.</p>
        {/if}

        {#if needsWork && r.kind === "pool" && node}
          <WaysToSatisfy {node} {ctx} />
        {/if}

        {#if r.notes.length > 0}<Notes notes={r.notes} />{/if}
      </div>
    </div>
    {#if isGroup && open}
      <div id={childrenId} class="mb-1 ml-[7px] border-l border-line pl-3">
        <RequirementList results={r.children} {ctx} depth={depth + 1} />
      </div>
    {/if}
  </li>
{/if}
