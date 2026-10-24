<script lang="ts">
  // The course card's "Aggregate Data" well: what the averages cover, then the three FCE boxes.
  import { termName } from "$lib/core/fce";
  import type { FceAggregate } from "./fce-aggregate";
  import FceStats from "./FceStats.svelte";

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
  <FceStats stats={aggregate} variant="plain" class="mt-3" />
</section>
