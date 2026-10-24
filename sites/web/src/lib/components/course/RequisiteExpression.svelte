<script lang="ts">
  import type { Requisite } from "$lib/core/catalog-types";
  import { codeOf } from "$lib/core/requisite-format";
  import { coursePath } from "$lib/core/routes";
  import CourseChip from "$lib/components/ui/CourseChip.svelte";
  import type { RequisiteStatus } from "$lib/components/ui/StatusChip.svelte";

  let {
    req,
    status,
    href,
    title,
    termId,
    size = "md",
    class: className = "",
  }: {
    /** The expression, from the catalog entry's `prereqs`. */
    req: Requisite;
    /** How the student stands on each course; omit to show plain course chips. */
    status?: (courseId: string) => RequisiteStatus | undefined;
    /** Where each course chip links; defaults to its course page. */
    href?: (courseId: string) => string;
    /** Course titles for the chips' tooltips and accessible names. */
    title?: (courseId: string) => string | undefined;
    /** Semester for the default course links. */
    termId?: string | null;
    size?: "sm" | "md";
    class?: string;
  } = $props();

  const word = (node: Requisite): string =>
    node.type === "all" ? "and" : node.type === "any" ? "or" : "";
  const linkOf = (id: string): string => href?.(id) ?? coursePath(id, termId);
  // A top-level "all of" reads as one row per requirement (the Figma popover); anything else is one line.
  const rows = $derived(req.type === "all" && req.items.length > 1 ? req.items : [req]);
</script>

{#snippet joiner(text: string)}
  <span class="px-1 text-body-2 text-ink">{text}</span>
{/snippet}

{#snippet node(item: Requisite, nested: boolean)}
  {#if item.type === "course"}
    <span class="inline-flex items-center gap-1" title={title?.(item.id)}>
      <CourseChip code={codeOf(item.id)} href={linkOf(item.id)} status={status?.(item.id)} {size} />
      {#if item.minGrade}<span class="text-label-md text-muted">min {item.minGrade}</span>{/if}
    </span>
  {:else if item.type === "text"}
    <span class="text-body-2 text-ink italic">{item.text}</span>
  {:else if item.items.length === 1}
    {@render node(item.items[0], nested)}
  {:else}
    {#if nested}<span aria-hidden="true" class="px-1 text-body-2 text-ink">(</span>{/if}
    {#if item.type === "atLeast"}<span class="pr-1 text-body-2 text-muted">at least {item.n} of</span>{/if}
    {#each item.items as child, index (index)}
      {#if index > 0}{@render joiner(item.type === "atLeast" ? "," : word(item))}{/if}
      {@render node(child, true)}
    {/each}
    {#if nested}<span aria-hidden="true" class="px-1 text-body-2 text-ink">)</span>{/if}
  {/if}
{/snippet}

<div class="flex min-w-0 flex-col gap-1.5 {className}">
  {#each rows as row, index (index)}
    <div class="flex min-w-0 flex-wrap items-center gap-y-1.5">
      {@render node(row, rows.length > 1)}
      {#if index < rows.length - 1}{@render joiner("and")}{/if}
    </div>
  {/each}
</div>
