<script lang="ts">
  import { untrack } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { ChevronLeft, SearchX, UserX } from "@lucide/svelte";
  import FilterButton from "$lib/components/layout/FilterButton.svelte";
  import RailGroup from "$lib/components/layout/RailGroup.svelte";
  import RailLayout from "$lib/components/layout/RailLayout.svelte";
  import { getSession } from "$lib/components/shell/session";
  import SignInPrompt from "$lib/components/shell/SignInPrompt.svelte";
  import Title from "$lib/components/shell/Title.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import MultiSelect from "$lib/components/ui/MultiSelect.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
  import Spinner from "$lib/components/ui/Spinner.svelte";
  import Switch from "$lib/components/ui/Switch.svelte";
  import FceRowsTable from "$lib/components/instructor/FceRowsTable.svelte";
  import FceStats from "$lib/components/instructor/FceStats.svelte";
  import { courseCode } from "$lib/components/instructor/format";
  import { catalog, resource } from "$lib/data";
  import { departmentName } from "$lib/core/departments";
  import type { Instructor } from "$lib/core/data";
  import type { AllInstructor } from "$lib/core/instructors/allInstructors";
  import { instructorPath, instructorsPath } from "$lib/core/routes";
  import { device, lastOrigin } from "$lib/state";
  import { ALL_SEMESTERS, isAllSemesters, resolveTerm } from "$lib/state/term-scope";
  import AggregateFceSettings from "./AggregateFceSettings.svelte";
  import { aggregateChoices } from "./aggregate-settings.svelte";
  import { loadAllInstructors, loadCourseLookup, semesterOptions } from "./data";
  import EvaluationHistory from "./EvaluationHistory.svelte";
  import { aggregate, loadInstructorFce, windowRows } from "./fce-aggregate";
  import TeachingHistory from "./TeachingHistory.svelte";
  import TeachingSections from "./TeachingSections.svelte";

  let { slug }: { slug: string } = $props();

  const session = getSession();
  const signedIn = $derived(session.status === "signed-in");
  const termParam = $derived(page.url.searchParams.get("term"));
  const allMode = $derived(isAllSemesters(termParam));

  const terms = resource(() => catalog.terms());
  const termId = $derived(terms.value ? resolveTerm(termParam, terms.value, device.lastTerm) : null);
  const term = resource(() => (signedIn && !allMode && termId !== null ? catalog.term(termId) : null));
  const all = resource(() => (signedIn && allMode ? loadAllInstructors() : null));
  const lookup = resource(() => (signedIn ? loadCourseLookup() : null));

  const inst = $derived<Instructor | AllInstructor | undefined>(
    allMode ? all.value?.bySlug.get(slug) : term.value?.instructorBySlug.get(slug),
  );
  const ready = $derived(allMode ? all.value !== undefined : term.value !== undefined);
  const fce = resource(() => (inst === undefined ? null : loadInstructorFce(catalog, inst.name)));

  // Which of their courses the aggregate and the table count; none chosen counts all of them.
  let courses = $state<string[]>([]);
  let everySemester = $state(false);
  const courseOptions = $derived(
    (fce.value?.summary.courses ?? [])
      .map((course) => ({
        value: course.courseId,
        label: courseCode(course.courseId),
        description: titleOf(course.courseId),
        keywords: titleOf(course.courseId),
      }))
      .toSorted((a, b) => a.label.localeCompare(b.label)),
  );
  const windowed = $derived(fce.value ? windowRows(fce.value.rows, aggregateChoices.value, courses) : []);
  const stats = $derived(fce.value ? aggregate(windowed) : null);
  const tableRows = $derived.by(() => {
    if (!everySemester) return windowed;
    const chosen = new Set(courses);
    return (fce.value?.rows ?? []).filter((row) => chosen.size === 0 || chosen.has(row.courseId));
  });

  // "All instructors" returns to the list as it was, when that's where the student came from.
  const back = untrack(() => {
    const origin = lastOrigin();
    return origin !== null && /^\/instructors\/?(?:\?|$)/u.test(origin) ? origin : instructorsPath(termParam);
  });

  function titleOf(courseId: string): string | undefined {
    return lookup.value?.title(courseId) ?? term.value?.courseById.get(courseId)?.title;
  }

  function setTerm(next: string): void {
    void goto(instructorPath(slug, next), { replaceState: true, noScroll: true, keepFocus: true });
  }
</script>

<Title value={inst?.name ?? "Instructor"} />

<RailLayout storageKey="instructors" filterCount={courses.length} backLabel="Instructor">
  {#snippet rail()}
    <RailGroup title="Aggregate FCEs">
      <AggregateFceSettings />
    </RailGroup>
    <RailGroup title="Courses">
      <MultiSelect
        options={courseOptions}
        bind:value={courses}
        label="Courses to count"
        placeholder={fce.value ? "All courses" : "No courses evaluated"}
        searchPlaceholder="Search courses"
        disabled={courseOptions.length === 0}
      />
      <p class="-mt-4 text-label-md font-normal text-muted">
        Count only some of their courses in the aggregate and the table.
      </p>
    </RailGroup>
  {/snippet}

  <div class="mb-4 flex items-center justify-between gap-3">
    <a
      href={back}
      class="inline-flex min-h-10 items-center gap-1 rounded-full pr-2 text-label-lg text-muted hover:text-ink"
    >
      <ChevronLeft aria-hidden="true" class="size-5" />All instructors
    </a>
    {#if signedIn}<FilterButton />{/if}
  </div>

  {#if session.status === "loading"}
    <div class="flex justify-center py-16"><Spinner size="lg" label="Checking your session" /></div>
  {:else if !signedIn}
    <SignInPrompt
      title="Sign in to see this instructor"
      description="Who teaches each course and their course evaluations come from CMU systems, so they're only shown to signed-in CMU students."
    />
  {:else if term.status === "forbidden" || all.status === "forbidden" || fce.status === "forbidden"}
    <EmptyState
      icon={UserX}
      title="Instructors need a CMU account"
      description="You're signed in with an account that isn't a CMU one. Sign out, then sign in with your Andrew ID."
    />
  {:else if term.status === "error" || all.status === "error"}
    <EmptyState
      icon={SearchX}
      title="This instructor didn't load"
      description="The course catalog couldn't be reached. Check your connection, then reload the page."
    >
      <Button onclick={() => location.reload()}>Reload</Button>
    </EmptyState>
  {:else if !ready}
    <div aria-busy="true" class="flex flex-col gap-4">
      <span class="sr-only" aria-live="polite">Loading the instructor</span>
      <Skeleton class="h-72 w-full rounded-card" />
      <Skeleton class="h-40 w-full rounded-card" />
    </div>
  {:else if inst === undefined}
    <Card padding="lg">
      <EmptyState
        icon={SearchX}
        headingLevel={1}
        title="No instructor at this link"
        description={allMode
          ? `Nobody named "${slug.replaceAll("-", " ")}" teaches in any semester here or appears in course evaluations.`
          : `Nobody named "${slug.replaceAll("-", " ")}" teaches in ${term.value?.meta.name ?? "this semester"}. They may teach in another semester.`}
      >
        <div class="flex flex-wrap justify-center gap-2">
          {#if !allMode}<Button href={instructorPath(slug, ALL_SEMESTERS)}>Look in all semesters</Button>{/if}
          <Button variant="secondary" href={instructorsPath(termParam)}>Browse instructors</Button>
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
          {#if terms.value}
            <Select
              label="Semester"
              variant="compact"
              options={semesterOptions(terms.value)}
              value={allMode ? ALL_SEMESTERS : (termId ?? "")}
              onchange={(event) => setTerm(event.currentTarget.value)}
              class="w-full sm:w-44"
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
                >(data from {stats?.semesters ?? 0} {stats?.semesters === 1 ? "semester" : "semesters"}{courses.length > 0
                  ? `, ${courses.length} ${courses.length === 1 ? "course" : "courses"}`
                  : ""})</span
              >
            </h2>
            <FceStats {stats} loading={fce.value === undefined} variant="plain" />
          </section>
          <div class="mt-4 mb-2 flex flex-wrap items-center justify-between gap-2">
            <h2 class="text-heading-6 text-strong">Evaluations by section</h2>
            <Switch label="Every semester" bind:checked={everySemester} disabled={fce.value === undefined} />
          </div>
          {#if fce.value === undefined}
            <Skeleton class="h-40 w-full rounded-card" />
          {:else}
            <FceRowsTable
              rows={tableRows}
              caption="Course evaluations for {inst.name}"
              empty={everySemester
                ? "No evaluations for these courses"
                : "No evaluations in the chosen semesters and seasons. Turn on Every semester to see them all."}
            />
          {/if}
        {/if}
      </Card>

      {#if allMode}
        <TeachingHistory inst={inst as AllInstructor} lookup={lookup.value} />
      {:else if term.value}
        <TeachingSections {inst} term={term.value} />
      {/if}

      {#if fce.value}
        <EvaluationHistory fce={fce.value.summary} title={titleOf} term={allMode ? null : termId} />
      {/if}
    </div>
  {/if}
</RailLayout>
