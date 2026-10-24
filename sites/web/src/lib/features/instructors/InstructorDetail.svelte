<script lang="ts">
  import { goto } from "$app/navigation";
  import { SearchX } from "@lucide/svelte";
  import Title from "$lib/components/shell/Title.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
  import Switch from "$lib/components/ui/Switch.svelte";
  import FceRowsTable from "$lib/components/instructor/FceRowsTable.svelte";
  import FceStats from "$lib/components/instructor/FceStats.svelte";
  import { departmentName } from "$lib/core/departments";
  import type { AllInstructor } from "$lib/core/instructors/allInstructors";
  import { instructorPath } from "$lib/core/routes";
  import { ALL_SEMESTERS } from "$lib/state/term-scope";
  import { semesterOptions } from "./data";
  import EvaluationHistory from "./EvaluationHistory.svelte";
  import { useInstructorsScope } from "./scope.svelte";
  import TeachingHistory from "./TeachingHistory.svelte";
  import TeachingSections from "./TeachingSections.svelte";

  // `/instructors/<slug>`: the Figma's instructor card (aggregate data and evaluations by section), then
  // Courseweave's teaching table or teaching history and evaluation history.
  const scope = useInstructorsScope();
  const inst = $derived(scope.selected);
  const fce = $derived(scope.fce);
  const tableRows = $derived.by(() => {
    if (!scope.everySemester) return scope.windowed;
    const chosen = new Set(scope.courses);
    return (fce.value?.rows ?? []).filter((row) => chosen.size === 0 || chosen.has(row.courseId));
  });

  function setTerm(next: string): void {
    const url = new URL(location.href);
    url.searchParams.set("term", next);
    void goto(url, { replaceState: true, noScroll: true, keepFocus: true });
  }
</script>

<Title value={inst?.name ?? "Instructor"} />

{#if !scope.signedIn}
  <!-- The layout asks the visitor to sign in. -->
{:else if !scope.ready}
  <div aria-busy="true" class="flex flex-col gap-4">
    <span class="sr-only" aria-live="polite">Loading the instructor</span>
    <Skeleton class="h-72 w-full rounded-card" />
    <Skeleton class="h-40 w-full rounded-card" />
  </div>
{:else if inst === undefined}
  {@const slug = scope.slug ?? ""}
  <Card padding="lg">
    <EmptyState
      icon={SearchX}
      headingLevel={1}
      title="No instructor at this link"
      description={scope.allMode
        ? `Nobody named "${slug.replaceAll("-", " ")}" teaches in any semester here or appears in course evaluations.`
        : `Nobody named "${slug.replaceAll("-", " ")}" teaches in ${scope.scopeName}. They may teach in another semester.`}
    >
      <div class="flex flex-wrap justify-center gap-2">
        {#if !scope.allMode}<Button href={instructorPath(slug, ALL_SEMESTERS)}>Look in all semesters</Button>{/if}
        <Button variant="secondary" href={scope.listHref}>Browse instructors</Button>
      </div>
    </EmptyState>
  </Card>
{:else}
  <div class="flex flex-col gap-10">
    <Card as="article" padding="lg" aria-labelledby="instructor-name">
      <div class="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div class="min-w-0">
          <h1 id="instructor-name" class="text-heading-4 text-ink">{inst.name}</h1>
          {#if inst.depts.length > 0}
            <p class="mt-1 text-body-1 text-muted">{inst.depts.map((code) => departmentName(code)).join(", ")}</p>
          {/if}
        </div>
        {#if scope.terms.value && !scope.split}
          <Select
            label="Semester"
            variant="compact"
            options={semesterOptions(scope.terms.value)}
            value={scope.allMode ? ALL_SEMESTERS : (scope.termId ?? "")}
            onchange={(event) => setTerm(event.currentTarget.value)}
            class="w-full sm:w-44 [&_select]:max-sm:h-10"
          />
        {/if}
      </div>

      {#if fce.status === "error"}
        <p class="mt-6 text-body-2 text-danger" role="alert">
          Course evaluations didn't load. Check your connection and reload the page.
        </p>
      {:else if fce.value === null}
        <p class="mt-6 rounded-card bg-surface-2 px-4 py-3 text-body-2 text-muted">
          No course evaluations match {inst.name}. New instructors, and anyone whose name is spelled differently in
          FCEs, have none.
        </p>
      {:else}
        <section aria-labelledby="aggregate-heading" class="mt-6 rounded-card bg-surface-2 p-4">
          <h2 id="aggregate-heading" class="mb-3 text-body-1 text-strong">
            Aggregate Data <span class="text-body-2 text-muted"
              >(data from {scope.stats?.semesters ?? 0}
              {scope.stats?.semesters === 1 ? "semester" : "semesters"}{scope.courses.length > 0
                ? `, ${scope.courses.length} ${scope.courses.length === 1 ? "course" : "courses"}`
                : ""})</span
            >
          </h2>
          <FceStats stats={scope.stats} loading={fce.value === undefined} variant="plain" />
        </section>
        <div class="mt-4 mb-2 flex flex-wrap items-center justify-between gap-2">
          <h2 class="text-heading-6 text-strong">Evaluations by section</h2>
          <Switch label="Every semester" bind:checked={scope.everySemester} disabled={fce.value === undefined} />
        </div>
        {#if fce.value === undefined}
          <Skeleton class="h-40 w-full rounded-card" />
        {:else}
          <FceRowsTable
            rows={tableRows}
            caption="Course evaluations for {inst.name}"
            empty={scope.everySemester
              ? "No evaluations for these courses"
              : "No evaluations in the chosen semesters and seasons. Turn on Every semester to see them all."}
          />
        {/if}
      {/if}
    </Card>

    {#if scope.allMode}
      <TeachingHistory inst={inst as AllInstructor} lookup={scope.lookup.value} />
    {:else if scope.term.value}
      <TeachingSections {inst} term={scope.term.value} />
    {/if}

    {#if fce.value}
      <EvaluationHistory
        fce={fce.value.summary}
        title={(id) => scope.titleOf(id)}
        term={scope.allMode ? null : scope.termId}
      />
    {/if}
  </div>
{/if}
