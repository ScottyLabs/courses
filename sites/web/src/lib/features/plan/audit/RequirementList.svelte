<script lang="ts">
  // A requirement's children, with the alternatives it doesn't need tucked behind a toggle.
  import { ChevronRight } from "@lucide/svelte";
  import type { RequirementResult } from "$lib/core/plan-types";
  import RequirementRow from "./RequirementRow.svelte";
  import type { TreeContext } from "./tree";

  let { results, ctx, depth }: { results: RequirementResult[]; ctx: TreeContext; depth: number } =
    $props();

  let showOthers = $state(false);
  const visible = $derived(results.filter((r) => r.skipped !== true));
  const skipped = $derived(results.filter((r) => r.skipped === true));
</script>

<ul class="flex flex-col">
  {#each visible as r (r.nodeId)}
    <RequirementRow {r} {ctx} {depth} />
  {/each}
  {#if skipped.length > 0}
    <li>
      <button
        type="button"
        aria-expanded={showOthers}
        onclick={() => (showOthers = !showOthers)}
        class="flex min-h-10 items-center gap-1.5 rounded-tag px-1 text-label-md text-muted hover:text-ink md:min-h-8"
      >
        <ChevronRight aria-hidden="true" class="size-3.5 transition-transform {showOthers ? 'rotate-90' : ''}" />
        {showOthers ? "Hide" : "Show"}
        {skipped.length} other option{skipped.length === 1 ? "" : "s"} not needed
      </button>
      {#if showOthers}
        <ul class="flex flex-col opacity-75">
          {#each skipped as r (r.nodeId)}
            <RequirementRow {r} {ctx} {depth} />
          {/each}
        </ul>
      {/if}
    </li>
  {/if}
</ul>
