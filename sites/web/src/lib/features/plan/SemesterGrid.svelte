<script lang="ts">
  // The plan's semesters in academic-year rows, with incoming credit above them and buttons to add a summer, a
  // removed semester or the next one. Courseweave's features/degree/SemesterGrid.tsx.
  import { Plus } from "@lucide/svelte";
  import { academicYearLabel } from "$lib/core/plan";
  import { requirementsByCourse } from "$lib/core/plan-table";
  import { termLabel } from "$lib/core/requisites";
  import { plans } from "$lib/state";
  import { planPage } from "./context";
  import { gridRows } from "./grid";
  import IncomingCredit from "./IncomingCredit.svelte";
  import TermCard from "./TermCard.svelte";

  const page = planPage();
  const table = $derived(page.layout === "table");
  const requirements = $derived(table ? requirementsByCourse(page.audit) : undefined);
  const rows = $derived(gridRows(page.plan.startTerm, page.derived.columns));
  const addClass =
    "flex items-center justify-center gap-1.5 rounded-card border border-dashed border-line-strong px-3 text-label-lg text-ink transition-colors hover:border-ink hover:bg-surface";
</script>

<div class="@container flex flex-col gap-6">
  <IncomingCredit />
  {#each rows.years as year (year.key)}
    {@const units = year.slots.reduce((n, s) => n + (s.column?.totals.units ?? 0), 0)}
    <section aria-labelledby="year-{year.key}" class="flex flex-col gap-3">
      <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
        <h2 id="year-{year.key}" class="text-heading-5 text-strong">{academicYearLabel(year.key)}</h2>
        <span class="tabular text-label-md font-normal text-muted">{units} units</span>
        {#if !year.hasSummer && !table}
          <button
            type="button"
            onclick={() => plans.addTerm(page.plan.id, year.summer)}
            class="ml-auto inline-flex min-h-10 items-center gap-1 rounded-full px-3 text-label-md text-ink hover:bg-hover md:min-h-8"
          >
            <Plus aria-hidden="true" class="size-3.5" />
            Add {termLabel(year.summer)}
          </button>
        {/if}
      </div>
      <div
        class={table
          ? "flex flex-col gap-3"
          : `grid grid-cols-1 items-start gap-4 @3xl:grid-cols-2 ${year.hasSummer ? "@6xl:grid-cols-3" : ""}`}
      >
        {#each year.slots as slot (slot.term)}
          {#if slot.column}
            <TermCard column={slot.column} {requirements} />
          {:else}
            <button
              type="button"
              onclick={() => plans.addTerm(page.plan.id, slot.term)}
              class="{addClass} {table ? 'min-h-11' : 'min-h-28 self-stretch'}"
            >
              <Plus aria-hidden="true" class="size-4" />
              Add {termLabel(slot.term)}
            </button>
          {/if}
        {/each}
        {#if !year.hasSummer && table}
          <!-- Detailed layout: the summer goes after the year's spring, so that's where it's added. -->
          <button type="button" onclick={() => plans.addTerm(page.plan.id, year.summer)} class="{addClass} min-h-10">
            <Plus aria-hidden="true" class="size-3.5" />
            Add {termLabel(year.summer)}
          </button>
        {/if}
      </div>
    </section>
  {/each}
  {#if !rows.nextInLastYear}
    <button type="button" onclick={() => plans.addTerm(page.plan.id, rows.next)} class="{addClass} min-h-11">
      <Plus aria-hidden="true" class="size-4" />
      Add {termLabel(rows.next)}
    </button>
  {/if}
</div>
