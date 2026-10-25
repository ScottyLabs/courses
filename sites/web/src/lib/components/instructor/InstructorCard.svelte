<script lang="ts">
  import type { Snippet } from "svelte";
  import Card from "../ui/Card.svelte";
  import { catalog, resource } from "$lib/data";
  import { departmentName } from "$lib/core/departments";
  import { aggregateFce, selectFceRows } from "../course/fce-aggregate";
  import { fceOptions } from "../course/fce-options.svelte";
  import FceStats from "../course/FceStats.svelte";
  import FceTable from "../course/FceTable.svelte";
  import { loadInstructorFce, loadInstructorRows, type InstructorRow } from "./instructor-fce";

  let {
    name,
    depts,
    href,
    meta,
    fceCourses,
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
    /**
     * The courses with FCE rows for them, from the instructor list's document (null: no FCE record matches the name).
     * Without it the card looks them up in the FCE index first.
     */
    fceCourses?: readonly string[] | null;
    headingLevel?: 2 | 3;
    class?: string;
  } = $props();

  // FCE rows load once the card comes near the viewport, after the list has painted, so a long list only fetches
  // what is seen and the names show first.
  let near = $state(false);
  async function load(): Promise<{ rows: InstructorRow[] } | null> {
    if (fceCourses === undefined) return loadInstructorFce(catalog, name);
    if (fceCourses === null) return null;
    return { rows: await loadInstructorRows(catalog, name, fceCourses) };
  }
  const fce = resource(() => (near ? load() : null));
  const rows = $derived(fce.value === undefined || fce.value === null ? [] : selectFceRows(fce.value.rows, fceOptions.value));
  const stats = $derived(fce.value ? aggregateFce(rows) : null);
  const loading = $derived(!near || (fce.status === "loading" && fce.value === undefined));
  const headingId = $props.id();

  function observe(element: HTMLElement): () => void {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          observer.disconnect();
          // The next frame, once this one has painted.
          requestAnimationFrame(() => setTimeout(() => (near = true)));
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
        <a {href} data-instructor-preview="off" class="hover:text-accent-ink hover:underline">{name}</a>
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
        >(data from {stats?.terms.length ?? 0} {stats?.terms.length === 1 ? "semester" : "semesters"})</span
      >
    </p>
    <FceTable
      showCourse
      showInstructor={false}
      {rows}
      caption="Course evaluations for {name} in the aggregate's semesters"
      empty="No evaluations in the chosen semesters and seasons"
      class="mt-2"
    />
  {/if}
</Card>
