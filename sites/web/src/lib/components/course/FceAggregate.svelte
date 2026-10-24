<script lang="ts">
  import StarRating from "$lib/components/ui/StarRating.svelte";
  import StatBox from "$lib/components/ui/StatBox.svelte";
  import { termName } from "$lib/core/fce";
  import type { FceAggregate } from "./fce-aggregate";

  let {
    aggregate,
    heading = "Aggregate Data",
    class: className = "",
  }: {
    /** From aggregateFce(). */
    aggregate: FceAggregate;
    heading?: string;
    class?: string;
  } = $props();

  const count = $derived(aggregate.terms.length);
  const span = $derived(
    count === 0
      ? "no responses"
      : `data from ${count} ${count === 1 ? "semester" : "semesters"}, ${aggregate.responses.toLocaleString("en-US")} responses`,
  );
</script>

<section
  class="rounded-card bg-surface-2 px-3 pt-3.5 pb-4 sm:px-4 {className}"
  aria-label="{heading}, {span}"
>
  <h3 class="flex flex-wrap items-baseline gap-x-2 text-body-1 text-ink">
    {heading}
    <span
      class="text-label-md text-muted"
      title={aggregate.terms.map((term) => termName(term)).join(", ")}>({span})</span
    >
  </h3>
  <div class="mt-3 grid gap-2 sm:grid-cols-3">
    <StatBox
      variant="plain"
      label="Workload"
      value={aggregate.hours === null ? "-" : aggregate.hours.toFixed(2)}
      unit="hrs/wk"
    />
    <StatBox variant="plain" label="Teaching Rate">
      {#if aggregate.teachingRate === null}
        <span class="text-[20px] leading-7">-</span>
      {:else}
        <StarRating value={aggregate.teachingRate} label="Teaching rate" />
      {/if}
    </StatBox>
    <StatBox variant="plain" label="Course Rate">
      {#if aggregate.courseRate === null}
        <span class="text-[20px] leading-7">-</span>
      {:else}
        <StarRating value={aggregate.courseRate} label="Course rate" />
      {/if}
    </StatBox>
  </div>
</section>
