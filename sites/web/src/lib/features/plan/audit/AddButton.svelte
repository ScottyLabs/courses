<script lang="ts">
  // "Add": puts a suggested course in the plan, in the semester the plan suggests. With a mouse it waits over the end
  // of its row (a `group/leaf`) until the row is hovered or focused; on touch, where nothing hovers, it's a + in the
  // row.
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
  class="inline-flex h-10 shrink-0 items-center justify-center gap-1 rounded-full bg-paper text-label-md text-ink transition-[background-color,opacity] duration-150 hover:bg-line active:bg-pressed pointer-coarse:w-10 pointer-fine:absolute pointer-fine:top-1/2 pointer-fine:right-0 pointer-fine:h-7 pointer-fine:-translate-y-1/2 pointer-fine:px-2.5 pointer-fine:opacity-0 pointer-fine:ring-4 pointer-fine:ring-surface pointer-fine:group-focus-within/leaf:opacity-100 pointer-fine:group-hover/leaf:opacity-100"
>
  <Plus aria-hidden="true" class="size-3.5" />
  <span class="pointer-coarse:sr-only">Add</span>
</button>
