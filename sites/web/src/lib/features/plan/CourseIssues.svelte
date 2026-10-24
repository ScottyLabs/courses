<script lang="ts">
  // The requisite issues under a planned course, with "Mark requisites as waived" and "Undo waiver".
  import type { PlanCheck } from "$lib/core/plan";
  import { record } from "$lib/state";

  let { id, check, fallback }: { id: string; check: PlanCheck; fallback: string | null } = $props();

  const issues = $derived(check.check.issues.filter((i) => i.severity !== "info"));
  const TONE = { error: "text-danger", warning: "text-warn", info: "text-muted" } as const;
</script>

<div {id} class="mt-1 mb-1.5 ml-2 flex flex-col gap-1 border-l-2 border-line pl-2.5 text-label-md font-normal">
  {#if check.check.issues.length === 0 && fallback !== null}<p class="text-muted">{fallback}</p>{/if}
  {#each check.check.issues as issue, i (i)}
    <p class={TONE[issue.severity]}>{issue.message}</p>
  {/each}
  {#if issues.some((i) => i.kind === "prereq" || i.kind === "coreq")}
    <button
      type="button"
      onclick={() =>
        record.setOverride(check.courseId, { prereqs: true, coreqs: true, note: "Waived from the degree plan" })}
      class="min-h-10 self-start text-label-md text-accent-ink hover:underline md:min-h-7"
    >
      Mark requisites as waived
    </button>
  {/if}
  {#if check.check.waived.prereqs || check.check.waived.coreqs}
    <button
      type="button"
      onclick={() => record.setOverride(check.courseId, null)}
      class="min-h-10 self-start text-label-md text-accent-ink hover:underline md:min-h-7"
    >
      Undo waiver
    </button>
  {/if}
</div>
