<script lang="ts">
  import { ChevronRight, Star } from "@lucide/svelte";
  import StarRating from "../ui/StarRating.svelte";

  let {
    name,
    href,
    depts,
    detail,
    count,
    rating = null,
  }: {
    name: string;
    href: string;
    /** Department codes, as Courseweave's rows show them. */
    depts: readonly string[];
    /** A second-line note, such as the semesters they teach in. */
    detail?: string;
    /** "3 courses", "5 sections". */
    count: string;
    /** Recent teaching rating from course evaluations. */
    rating?: number | null;
  } = $props();
</script>

<a
  {href}
  class="group flex min-h-14 items-center gap-3 px-4 py-2 transition-colors hover:bg-hover focus-visible:-outline-offset-2 sm:px-5"
>
  <span class="min-w-0 flex-1">
    <span class="block truncate text-body-1 font-medium text-ink group-hover:text-accent-ink">{name}</span>
    <span class="flex min-w-0 gap-2 text-label-md font-normal text-muted">
      {#if depts.length > 0}<span class="shrink-0">{depts.join(", ")}</span>{/if}
      {#if rating !== null}
        <span class="inline-flex shrink-0 items-center gap-0.5 sm:hidden">
          <Star aria-hidden="true" class="size-3 fill-current" strokeWidth={0} />
          <span class="sr-only">Recent teaching rating</span>{rating.toFixed(2)}
        </span>
      {/if}
      {#if detail}<span class="truncate">{detail}</span>{/if}
    </span>
  </span>
  {#if rating !== null}
    <span class="hidden shrink-0 sm:block">
      <StarRating value={rating} size="sm" label="Recent teaching rating" />
    </span>
  {/if}
  <span class="tabular shrink-0 text-right text-label-md text-ink sm:w-24">{count}</span>
  <ChevronRight aria-hidden="true" class="size-4 shrink-0 text-muted" />
</a>
