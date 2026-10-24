<script lang="ts">
  // "Add": puts a suggested course in the plan, in the semester the plan suggests.
  import { Plus } from "@lucide/svelte";
  import { codeOf } from "$lib/core/requisite-format";
  import type { TreeContext } from "./tree";

  let { courseId, nodeId, ctx }: { courseId: string; nodeId: number; ctx: TreeContext } = $props();
</script>

<button
  type="button"
  onclick={() => ctx.onAddCourse?.(courseId, nodeId)}
  aria-label="Add {codeOf(courseId)} to your plan"
  title={ctx.offered(courseId)
    ? `Add ${codeOf(courseId)} to your plan`
    : `Add ${codeOf(courseId)} (not offered in ${ctx.termName})`}
  class="inline-flex h-10 shrink-0 items-center gap-1 rounded-full border border-line-strong px-2.5 text-label-md text-ink transition-colors hover:bg-hover md:h-7"
>
  <Plus aria-hidden="true" class="size-3.5" />
  Add
</button>
