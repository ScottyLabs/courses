<script lang="ts">
  // Courses that would satisfy an unmet pool requirement, offered ones first, each with "Add" (and drag).
  import { courseOptions } from "$lib/core/audit";
  import { fmt } from "$lib/core/degree/audit/format";
  import type { ReqNode } from "$lib/core/degree-types";
  import { codeOf } from "$lib/core/requisite-format";
  import AddButton from "./AddButton.svelte";
  import { dragNew, type TreeContext } from "./tree";

  let { node, ctx }: { node: ReqNode; ctx: TreeContext } = $props();

  const SHOWN = 8;
  const LIMIT = 120;
  let all = $state(false);

  const options = $derived.by(() => {
    if (ctx.catalog === null) return [];
    return courseOptions(node, ctx.catalog, LIMIT)
      .filter((id) => !ctx.inPlan.has(id))
      .toSorted((a, b) => Number(ctx.offered(b)) - Number(ctx.offered(a)) || a.localeCompare(b));
  });
  const shown = $derived(all ? options : options.slice(0, SHOWN));
  const offeredCount = $derived(options.filter((id) => ctx.offered(id)).length);
</script>

{#if options.length > 0}
  <div class="mt-2">
    <p class="text-label-md text-muted">
      Ways to satisfy{offeredCount > 0 ? `, ${offeredCount} offered in ${ctx.termName}` : ""}
    </p>
    <ul class="mt-1 flex flex-col">
      {#each shown as id (id)}
        {@const drag = dragNew(id, ctx)}
        {@const units = ctx.units(id)}
        <li class="flex min-h-11 items-center gap-2 md:min-h-8 {drag.draggable ? 'cursor-grab' : ''}" {...drag}>
          <a
            href={ctx.href(id)}
            draggable={drag.draggable ? "false" : undefined}
            class="tabular shrink-0 text-label-lg hover:text-accent-ink hover:underline {ctx.offered(id)
              ? 'text-strong'
              : 'text-muted'}"
          >
            {codeOf(id)}
          </a>
          <span class="min-w-0 flex-1 truncate text-body-2 text-ink" title={ctx.title(id)}>{ctx.title(id) ?? ""}</span>
          {#if units !== null}<span class="tabular shrink-0 text-label-md text-muted">{fmt(units)}u</span>{/if}
          {#if ctx.onAddCourse}<AddButton courseId={id} nodeId={node.id} {ctx} />{/if}
        </li>
      {/each}
    </ul>
    {#if options.length > SHOWN}
      <button
        type="button"
        onclick={() => (all = !all)}
        class="mt-1 min-h-10 text-label-md text-accent-ink hover:underline md:min-h-7"
      >
        {all ? "Show fewer" : `Show ${options.length - SHOWN} more${options.length >= LIMIT ? "+" : ""}`}
      </button>
    {/if}
  </div>
{/if}
