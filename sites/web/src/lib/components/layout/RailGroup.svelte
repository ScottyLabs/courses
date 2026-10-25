<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    count,
    summary,
    action,
    children,
    class: className = "",
  }: {
    /** Group heading (Figma "Aggregate FCEs"). Groups after the first get a divider above. */
    title?: string;
    /** A count after the heading, as a pill (the schedule's number of courses). */
    count?: number;
    /** Short lines under the heading, such as the schedule's totals. */
    summary?: Snippet;
    /** A small control at the end of the heading row, such as Expand. */
    action?: Snippet;
    children: Snippet;
    class?: string;
  } = $props();
</script>

<section
  class="border-line px-5 pb-8 not-first:border-t not-first:pt-8 sm:px-6 min-[90rem]:px-[31px] {className}"
>
  {#if title}
    <h2 class="flex items-center gap-2 text-heading-5 text-ink {summary ? 'mb-1' : 'mb-3'}">
      {title}
      {#if count !== undefined}
        <span class="tabular rounded-full bg-line px-1.5 text-label-md text-ink">{count}</span>
      {/if}
      {#if action}<span class="ml-auto text-body-2">{@render action()}</span>{/if}
    </h2>
  {/if}
  {#if summary}
    <div class="mb-4">{@render summary()}</div>
  {/if}
  <div class="flex flex-col gap-6">
    {@render children()}
  </div>
</section>
