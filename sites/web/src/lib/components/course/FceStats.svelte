<script lang="ts">
  // The Figma's three FCE boxes: workload, teaching rate and course rate, for course and instructor cards and pages.
  import { Star } from "@lucide/svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
  import StarRating from "$lib/components/ui/StarRating.svelte";
  import StatBox from "$lib/components/ui/StatBox.svelte";
  import { FCE_HOURS_HELP } from "$lib/core/fce";
  import type { FceAggregate } from "./fce-aggregate";
  import { trimNumber } from "./fce-format";

  let {
    stats,
    loading = false,
    variant = "outline",
    class: className = "",
  }: {
    /** The aggregate to show; null shows dashes. */
    stats: FceAggregate | null;
    /** Show placeholders while the rows load. */
    loading?: boolean;
    /** outline: bordered boxes beside the name (instructor list cards); plain: white boxes in the gray well. */
    variant?: "outline" | "plain";
    class?: string;
  } = $props();

  const ratings = $derived([
    { label: "Teaching Rate", value: stats?.teachingRate ?? null },
    { label: "Course Rate", value: stats?.courseRate ?? null },
  ]);
</script>

<!-- Three boxes in a row; on phones the stars shrink to one star and the number. -->
<div class="grid grid-cols-3 gap-2 sm:gap-3 {className}">
  {#if loading}
    {#each ["Workload", "Teaching Rate", "Course Rate"] as label (label)}
      <StatBox {label} {variant}><Skeleton class="my-1 h-5 w-full max-w-24" /></StatBox>
    {/each}
  {:else}
    <div title={FCE_HOURS_HELP} class="grid">
      <StatBox label="Workload" {variant}>
        {#if stats?.hours === null || stats?.hours === undefined}
          <span class="tabular text-[20px] leading-7" aria-label="No responses">&ndash;</span>
        {:else}
          <span class="flex flex-wrap items-baseline gap-x-1">
            <span class="tabular text-[20px] leading-7">{trimNumber(stats.hours)}</span>
            <span class="text-body-2 sm:text-body-1">hrs/wk</span>
          </span>
        {/if}
      </StatBox>
    </div>
    {#each ratings as box (box.label)}
      <StatBox label={box.label} {variant}>
        {#if box.value === null}
          <span class="tabular text-[20px] leading-7" aria-label="No responses">&ndash;</span>
        {:else}
          <span class="hidden sm:inline-flex"><StarRating value={box.value} label={box.label} /></span>
          <span class="inline-flex items-center gap-1 sm:hidden">
            <Star aria-hidden="true" class="size-4 fill-current" strokeWidth={0} />
            <span class="tabular text-[20px] leading-7">{trimNumber(box.value)}</span>
            <span class="sr-only">out of 5</span>
          </span>
        {/if}
      </StatBox>
    {/each}
  {/if}
</div>
