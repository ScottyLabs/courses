<script lang="ts">
  // A course counted toward a requirement: solid green when done, dashed when planned.
  import { fmt } from "$lib/core/degree/audit/format";
  import type { RequirementResult } from "$lib/core/plan-types";
  import { codeOf } from "$lib/core/requisite-format";
  import { assignedStatus } from "./status-text";
  import type { TreeContext } from "./tree";

  let { a, ctx }: { a: RequirementResult["assigned"][number]; ctx: TreeContext } = $props();

  const done = $derived(a.status === "completed" || a.status === "credit");
  const title = $derived(ctx.title(a.courseId));
</script>

<a
  href={ctx.href(a.courseId, a.term)}
  aria-label="{codeOf(a.courseId)}{title ? ` ${title}` : ''}, {assignedStatus(a)}"
  class="inline-flex min-h-10 items-center gap-1.5 rounded-tag border px-2 text-label-md transition-colors md:min-h-7 {done
    ? 'border-transparent bg-ok-bg text-strong hover:border-ok'
    : 'border-dashed border-line-strong bg-surface text-strong hover:border-ink'}"
>
  <span class="tabular font-semibold">{codeOf(a.courseId)}</span>
  <span class="font-normal text-muted">{assignedStatus(a)}</span>
  {#if a.units}<span class="tabular font-normal text-muted">{fmt(a.units)}u</span>{/if}
</a>
