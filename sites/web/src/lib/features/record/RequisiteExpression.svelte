<script lang="ts">
  // A requisite expression as course chips: a top-level "all of" becomes one row per requirement, alternatives read
  // inline ("21-127 or 15-151"), and chips show met, planned or missing when `ctx.status` is given. `ids` renders a
  // plain list of chips instead (corequisites, anti-requisites). Courseweave's RequisiteExpression and CourseChipList
  // (features/catalog/RequisiteExpression.tsx), on the design system's CourseChip.
  import CourseChip from "$lib/components/ui/CourseChip.svelte";
  import type { RequisiteStatus as ChipStatus } from "$lib/components/ui/StatusChip.svelte";
  import type { Requisite } from "$lib/core/catalog-types";
  import type { ChipContext, RequisiteStatus } from "$lib/core/catalog/chip-context";
  import { codeOf } from "$lib/core/requisite-format";

  let {
    req,
    ids,
    joinWord,
    ctx,
    class: className = "",
  }: {
    req?: Requisite;
    /** A plain list of courses instead of an expression. */
    ids?: string[];
    /** Word between the listed courses ("or"). */
    joinWord?: string;
    ctx: ChipContext;
    class?: string;
  } = $props();

  const CHIP: Record<RequisiteStatus, ChipStatus> = {
    met: "satisfied",
    planned: "planned",
    missing: "unsatisfied",
  };

  function chipStatus(id: string): ChipStatus | undefined {
    const status = ctx.status?.(id);
    return status === undefined ? undefined : CHIP[status];
  }
</script>

{#snippet chip(id: string, minGrade: string | undefined)}
  <CourseChip
    code={codeOf(id)}
    title={minGrade === undefined ? undefined : `min ${minGrade}`}
    status={chipStatus(id)}
    href={ctx.href(id)}
    size="md"
  />
{/snippet}

{#snippet joiner(word: string)}
  <span class="px-0.5 text-label-md text-muted">{word}</span>
{/snippet}

{#snippet inline(node: Requisite, nested: boolean)}
  {#if node.type === "course"}
    {@render chip(node.id, node.minGrade)}
  {:else if node.type === "text"}
    <span class="text-body-2 text-ink italic">{node.text}</span>
  {:else}
    {@const word = node.type === "all" ? "and" : node.type === "any" ? "or" : ""}
    <span
      class="inline-flex max-w-full flex-wrap items-center gap-1 {nested
        ? 'rounded-tag bg-surface-2 px-1 py-0.5 ring-1 ring-line'
        : ''}"
    >
      {#if node.type === "atLeast"}<span class="text-label-md text-muted">at least {node.n} of</span>{/if}
      {#each node.items as item, index (index)}
        {#if index > 0 && word !== ""}{@render joiner(word)}{/if}
        {@render inline(item, true)}
      {/each}
    </span>
  {/if}
{/snippet}

{#if ids !== undefined}
  <span class="inline-flex min-w-0 flex-wrap items-center gap-1 {className}">
    {#each ids as id, index (id)}
      {#if index > 0 && joinWord !== undefined}{@render joiner(joinWord)}{/if}
      {@render chip(id, undefined)}
    {/each}
  </span>
{:else if req !== undefined && req.type === "all" && req.items.length > 1}
  <ul class="flex min-w-0 flex-col gap-1.5 {className}" aria-label="All of these">
    {#each req.items as item, index (index)}
      <li class="flex min-w-0 flex-wrap items-center gap-x-1.5 gap-y-1">
        {#if item.type === "any"}<span class="text-label-md text-muted">One of</span>{/if}
        {@render inline(item, false)}
      </li>
    {/each}
  </ul>
{:else if req !== undefined}
  <div class="flex min-w-0 flex-wrap items-center gap-x-1.5 gap-y-1 {className}">
    {#if req.type === "any"}<span class="text-label-md text-muted">One of</span>{/if}
    {@render inline(req, false)}
  </div>
{/if}
