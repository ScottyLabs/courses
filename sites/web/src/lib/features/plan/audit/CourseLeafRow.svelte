<script lang="ts">
  // A requirement for one specific course: done, planned, or an "Add" (and drag) when it's still needed.
  import { Info } from "@lucide/svelte";
  import { displayState, STATE_LABEL } from "$lib/core/degree/audit/format";
  import type { RequirementResult } from "$lib/core/plan-types";
  import { codeOf } from "$lib/core/requisite-format";
  import AddButton from "./AddButton.svelte";
  import StateIcon from "./StateIcon.svelte";
  import { assignedStatus } from "./status-text";
  import { dragNew, type TreeContext } from "./tree";

  let { r, ctx }: { r: RequirementResult; ctx: TreeContext } = $props();

  const state = $derived(displayState(r));
  const id = $derived(r.courseId ?? "");
  const title = $derived(ctx.title(id));
  const a = $derived(r.assigned.at(0));
  const counted = $derived(a !== undefined && a.courseId !== id ? a.courseId : null);
  const done = $derived(state === "complete" || state === "complete-with-plan");
  const canDrag = $derived(a === undefined && !done && state !== "skipped" && !ctx.inPlan.has(id));
  const drag = $derived(canDrag ? dragNew(id, ctx) : {});
</script>

<li
  id="req-{r.nodeId}"
  class="flex min-h-11 items-center gap-2 border-t border-line py-1 first:border-t-0 md:min-h-9 {drag.draggable
    ? 'cursor-grab'
    : ''}"
  {...drag}
>
  <StateIcon {state} class="size-4" />
  <span class="sr-only">{STATE_LABEL[state]}</span>
  <!-- In a draggable row the link mustn't start its own drag: the row's drag adds the course. -->
  <a
    href={ctx.href(id, a?.term)}
    draggable={drag.draggable ? "false" : undefined}
    class="tabular shrink-0 text-label-lg text-strong hover:text-accent-ink hover:underline"
  >
    {codeOf(id)}
  </a>
  <span class="min-w-0 flex-1 truncate text-body-2 {state === 'skipped' ? 'text-muted' : 'text-ink'}" title={title}>
    {title ?? ""}{#if counted}<span class="text-muted"> (as {codeOf(counted)})</span>{/if}
  </span>
  {#if a}
    <span class="shrink-0 text-label-md text-muted">{assignedStatus(a)}</span>
  {:else if !done && state !== "skipped" && ctx.onAddCourse && !ctx.inPlan.has(id)}
    <AddButton courseId={id} nodeId={r.nodeId} {ctx} />
  {:else if !ctx.offered(id) && !done}
    <span class="shrink-0 text-label-md text-muted">Not offered in {ctx.termName}</span>
  {/if}
  {#if r.notes.length > 0}
    <span title={r.notes.join("\n")} class="grid size-7 shrink-0 place-items-center text-muted">
      <Info aria-hidden="true" class="size-3.5" />
      <span class="sr-only">{r.notes.join(". ")}</span>
    </span>
  {/if}
</li>
