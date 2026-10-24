<script lang="ts">
  import { ArrowRight, Check, ChevronDown, Info, X } from "@lucide/svelte";
  import Popover from "$lib/components/ui/Popover.svelte";
  import type { Requisite } from "$lib/core/catalog-types";
  import { codeOf } from "$lib/core/requisite-format";
  import { evaluateRequisite, requisiteCourses, type RequisiteStatus } from "$lib/core/requisites";
  import { profilePath } from "$lib/core/routes";
  import { catalogIndex } from "./data.svelte";
  import { termRequisites } from "./requisite-context.svelte";
  import RequisiteExpression from "./RequisiteExpression.svelte";

  let {
    courseId,
    termId,
    kind = "prereqs",
    class: className = "",
  }: {
    courseId: string;
    /** The semester the course would be taken in; what's planned before it counts as planned. */
    termId: string;
    /** Prerequisites, or corequisites (one of them before or alongside). */
    kind?: "prereqs" | "coreqs";
    class?: string;
  } = $props();

  const index = $derived(catalogIndex());
  const entry = $derived(index.value?.courses[courseId]);
  const expression = $derived.by((): Requisite | null => {
    if (entry === undefined) return null;
    if (kind === "prereqs") return entry.prereqs;
    const ids = entry.coreqs.filter((id) => id !== courseId);
    if (ids.length === 0) return null;
    if (ids.length === 1) return { type: "course", id: ids[0] ?? "" };
    return { type: "any", items: ids.map((id) => ({ type: "course" as const, id })) };
  });
  const requisites = $derived(termRequisites(termId));
  const ctx = $derived(requisites.ctx);
  // Nothing on record, planned or scheduled: every check would read "missing", which says nothing.
  const unchecked = $derived(ctx === null || ctx.credits.size === 0);
  const waived = $derived(ctx?.overrides[courseId]?.[kind] === true);
  const result = $derived.by((): RequisiteStatus | null => {
    if (expression === null || ctx === null) return null;
    return evaluateRequisite(expression, ctx, { allowConcurrent: kind === "coreqs" }).status;
  });
  // Unchecked, the trigger names the courses briefly: "15-122", "15-122 and 4 more".
  const brief = $derived.by(() => {
    const ids = requisiteCourses(expression);
    const first = ids[0];
    if (first === undefined) return "See details";
    return ids.length === 1 ? codeOf(first) : `${codeOf(first)} and ${ids.length - 1} more`;
  });
  const chipStatus = $derived(kind === "coreqs" ? requisites.coreqStatus : requisites.prereqStatus);
  const titleOf = (id: string): string | undefined => index.value?.courses[id]?.title;

  const looks = {
    met: { text: "All Satisfied", icon: Check, box: "bg-ok-bg text-ok" },
    planned: { text: "Planned", icon: ArrowRight, box: "bg-warn-bg text-warn" },
    missing: { text: "Not Satisfied", icon: X, box: "bg-paper text-ink" },
    unknown: { text: "Check details", icon: Info, box: "bg-paper text-ink" },
  } as const;
  const look = $derived(result === null ? null : looks[result]);
  const label = $derived(kind === "prereqs" ? "Prerequisites" : "Corequisites");
</script>

{#if index.status === "loading"}
  <span class="inline-block h-4 w-24 animate-pulse rounded-tag bg-line {className}" aria-hidden="true"></span>
{:else if expression === null}
  <span class="text-body-1 text-ink {className}">None</span>
{:else}
  <Popover label={label} placement="bottom-start" class="w-[min(90vw,26rem)] p-4">
    {#snippet trigger(props)}
      <button
        type="button"
        {...props}
        class="inline-flex max-w-full min-w-0 items-center gap-1.5 rounded-tag py-0.5 pr-1 text-left text-body-1 text-ink hover:bg-hover {className}"
      >
        {#if waived}
          <span class="grid size-4 shrink-0 place-items-center rounded-[3px] bg-ok-bg text-ok">
            <Check aria-hidden="true" class="size-3" strokeWidth={3} />
          </span>
          <span class="truncate">Waived</span>
        {:else if unchecked || look === null}
          <span class="truncate">{brief}</span>
        {:else}
          <span class="grid size-4 shrink-0 place-items-center rounded-[3px] {look.box}">
            <look.icon aria-hidden="true" class="size-3" strokeWidth={3} />
          </span>
          <span class="truncate">{look.text}</span>
        {/if}
        <span class="sr-only">, show {label.toLowerCase()}</span>
        <ChevronDown aria-hidden="true" class="size-4 shrink-0 text-muted" />
      </button>
    {/snippet}
    <p class="mb-2 text-label-lg text-ink">
      {kind === "prereqs" ? "Prerequisites" : "Corequisites, before or in the same semester"}
    </p>
    <RequisiteExpression
      req={expression}
      status={unchecked ? undefined : (chipStatus ?? undefined)}
      title={titleOf}
      {termId}
    />
    {#if kind === "prereqs" && entry?.prereqText}
      <p class="mt-2 text-label-md text-muted">Catalog: {entry.prereqText}</p>
    {/if}
    <p class="mt-3 border-t border-line pt-2 text-label-md text-muted">
      {#if unchecked}
        <a href={profilePath("taken")} class="text-accent-ink underline underline-offset-2">Add the courses you've taken</a>
        to see which of these you've met.
      {:else}
        Green is done and yellow is planned, from your
        <a href={profilePath("taken")} class="text-accent-ink underline underline-offset-2">courses taken</a>, degree plan
        and schedules.
      {/if}
    </p>
  </Popover>
{/if}
