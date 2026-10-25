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

<!-- In a narrow panel (the column beside the semesters, phones) the title takes a second line under the number, so
     the number and Add share the first; from 32rem it sits between them on one line. -->
<li
  id="req-{r.nodeId}"
  class="flex min-h-11 flex-wrap items-center gap-x-2 border-t border-line py-1 first:border-t-0 md:min-h-9 {drag.draggable
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
  <span
    class="order-last min-w-0 basis-full truncate pb-1 pl-6 text-body-2 @lg/req:order-none @lg/req:flex-1 @lg/req:basis-auto @lg/req:pb-0 @lg/req:pl-0 {state ===
    'skipped'
      ? 'text-muted'
      : 'text-ink'}"
    {title}
  >
    {title ?? ""}{#if counted}<span class="text-muted"> (as {codeOf(counted)})</span>{/if}
  </span>
  <span class="ml-auto flex shrink-0 items-center gap-1 @lg/req:ml-0">
    {#if a}
      <span class="text-label-md text-muted">{assignedStatus(a)}</span>
    {:else if !done && state !== "skipped" && ctx.onAddCourse && !ctx.inPlan.has(id)}
      <AddButton courseId={id} nodeId={r.nodeId} {ctx} />
    {:else if !ctx.offered(id) && !done}
      <span class="text-label-md text-muted">Not offered in {ctx.termName}</span>
    {/if}
    {#if r.notes.length > 0}
      <span title={r.notes.join("\n")} class="grid size-7 shrink-0 place-items-center text-muted">
        <Info aria-hidden="true" class="size-3.5" />
        <span class="sr-only">{r.notes.join(". ")}</span>
      </span>
    {/if}
  </span>
</li>
