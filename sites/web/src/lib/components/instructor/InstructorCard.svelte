<script lang="ts">
  import type { Snippet } from "svelte";
  import Card from "../ui/Card.svelte";
  import { catalog, resource } from "$lib/data";
  import { departmentName } from "$lib/core/departments";
  import { aggregateChoices } from "$lib/features/instructors/aggregate-settings.svelte";
  import { aggregate, loadInstructorFce, windowRows } from "$lib/features/instructors/fce-aggregate";
  import FceRowsTable from "./FceRowsTable.svelte";
  import FceStats from "./FceStats.svelte";

  let {
    name,
    depts,
    href,
    meta,
    headingLevel = 2,
    class: className = "",
  }: {
    /** Schedule-style name, "David Kosbie". */
    name: string;
    /** Department codes. */
    depts: readonly string[];
    /** The instructor's page. */
    href: string;
    /** A line under the departments, such as the courses they teach this semester. */
    meta?: Snippet;
    headingLevel?: 2 | 3;
    class?: string;
  } = $props();

  // FCE rows load once the card comes near the viewport, so a long list only fetches what is seen.
  let near = $state(false);
  const fce = resource(() => (near ? loadInstructorFce(catalog, name) : null));
  const rows = $derived(fce.value === undefined || fce.value === null ? [] : windowRows(fce.value.rows, aggregateChoices.value));
  const stats = $derived(fce.value ? aggregate(rows) : null);
  const loading = $derived(!near || (fce.status === "loading" && fce.value === undefined));
  const headingId = $props.id();

  function observe(element: HTMLElement): () => void {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          near = true;
          observer.disconnect();
        }
      },
      { rootMargin: "600px 0px" },
    );
    observer.observe(element);
    return () => observer.disconnect();
  }
</script>

<Card as="article" padding="lg" aria-labelledby={headingId} class={className} {@attach observe}>
  <div class="flex flex-col gap-4 xl:flex-row xl:items-start xl:gap-10">
    <div class="min-w-0 xl:w-[200px] xl:shrink-0">
      <svelte:element this={`h${headingLevel}`} id={headingId} class="text-heading-4 text-ink">
        <a {href} class="hover:text-accent-ink hover:underline">{name}</a>
      </svelte:element>
      {#if depts.length > 0}
        <p class="mt-1 text-body-1 text-muted">{depts.map((code) => departmentName(code)).join(", ")}</p>
      {/if}
      {@render meta?.()}
    </div>
    <FceStats {stats} {loading} class="min-w-0 xl:flex-1" />
  </div>

  {#if fce.status === "error"}
    <p class="mt-6 text-body-2 text-danger" role="alert">
      Course evaluations didn't load. Check your connection and reload the page.
    </p>
  {:else if !loading && fce.value === null}
    <p class="mt-6 rounded-card bg-surface-2 px-4 py-3 text-body-2 text-muted">
      No course evaluations match this name. New instructors, and anyone whose name is spelled differently in FCEs,
      have none.
    </p>
  {:else if !loading}
    <p class="mt-6 rounded-card bg-surface-2 px-4 py-3 text-body-1 text-strong">
      Aggregate Data <span class="text-body-2 text-muted"
        >(data from {stats?.semesters ?? 0} {stats?.semesters === 1 ? "semester" : "semesters"})</span
      >
    </p>
    <FceRowsTable
      {rows}
      caption="Course evaluations for {name} in the aggregate's semesters"
      empty="No evaluations in the chosen semesters and seasons"
      class="mt-2"
    />
  {/if}
</Card>
