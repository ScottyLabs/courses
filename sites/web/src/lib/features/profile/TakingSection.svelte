<script lang="ts">
  // Profile "Currently Taking": this semester's courses from the academic record, with the sections and times chosen
  // in the semester's active schedule when there is one. Figma "Profile Page (Currently Taking)".
  import { Calendar1, CalendarPlus, Plus } from "@lucide/svelte";
  import { MediaQuery } from "svelte/reactivity";
  import { profileBand } from "$lib/components/layout/profile-band.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import { toast } from "$lib/components/ui/toast.svelte";
  import ViewToggle, { storedView, type ListView } from "$lib/components/ui/ViewToggle.svelte";
  import type { Option } from "$lib/components/ui/types";
  import { scheduleCourseIds } from "$lib/core/record/context";
  import {
    ALL_FILTERS,
    currentlyTaking,
    departmentOptions,
    levelOptions,
    matchesFilters,
    missingFromRecord,
    type ListFilters,
  } from "$lib/core/record/taken-list";
  import { codeOf } from "$lib/core/requisite-format";
  import { termLabel } from "$lib/core/requisites";
  import { coursePath, profilePath, schedulesPath } from "$lib/core/routes";
  import { entryUnits } from "$lib/core/state/record";
  import { buildScheduleView } from "$lib/core/state/schedule-view";
  import { catalog, resource } from "$lib/data";
  import Spinner from "$lib/components/ui/Spinner.svelte";
  import { personal, record, schedules } from "$lib/state";
  import { recordData } from "$lib/features/record/record-data.svelte";
  import SioImportButton from "$lib/features/record/SioImportButton.svelte";
  import TakingCard from "$lib/features/record/TakingCard.svelte";

  // The profile page passes the section's title; its band shows it.
  let { title }: { title: string } = $props();

  const data = recordData();
  const termId = data.current;
  const semester = termLabel(termId);
  // Sections need the semester's schedule of classes, when it's built.
  const built = $derived(data.terms.some((t) => t.id === termId));
  const term = resource(() => (built ? catalog.term(termId) : null));

  let view = $state<ListView>(storedView("profile-taking") ?? "cards");
  let filters = $state<ListFilters>({ ...ALL_FILTERS });
  // Phones get the taller field selects (40px tap targets); wider screens the Figma's compact filter boxes.
  const wide = new MediaQuery("(min-width: 40rem)");

  const taking = $derived(currentlyTaking(record.entries, termId));
  const schedule = $derived(schedules.active(termId));
  const scheduleView = $derived(term.value === undefined ? null : buildScheduleView(schedule, term.value));
  const byCourse = $derived(new Map((scheduleView?.entries ?? []).map((e) => [e.course.id, e])));
  const missing = $derived(
    schedule === null ? [] : missingFromRecord(record.entries, termId, scheduleCourseIds(schedule)),
  );

  const facts = (id: string): ReturnType<typeof data.facts> => data.facts(id);
  const shown = $derived(taking.filter((e) => matchesFilters(e, filters, facts)));
  const deptChoices = $derived<Option[]>([
    { value: "", label: "All departments" },
    ...departmentOptions(taking, facts),
  ]);
  const levelChoices = $derived<Option[]>([{ value: "", label: "All levels" }, ...levelOptions(taking, facts)]);
  const units = $derived(
    taking.reduce((n, e) => n + entryUnits(e, e.courseId === null ? null : data.lookup(e.courseId).units), 0),
  );
  const fmt = (n: number): string => (Number.isInteger(n) ? String(n) : n.toFixed(1));

  // The filters and the cards/list switch sit in the section's title band, as in the Figma.
  profileBand(() => ({ filters: personal.mode !== "loading" && taking.length > 0 ? takingFilters : undefined }));

  function addMissing(): void {
    const count = missing.length;
    record.addEntries(missing.map((courseId) => ({ courseId, source: "taken" as const, term: termId })));
    toast(`Added ${count} ${count === 1 ? "course" : "courses"} to your record for ${semester}.`, {
      kind: "success",
    });
  }
</script>

{#snippet takingFilters()}
  <div class="grid w-full grid-cols-2 gap-3 sm:flex sm:flex-wrap sm:items-center">
    <Select
      label="Department"
      variant={wide.current ? "compact" : "field"}
      class="col-span-2 sm:w-52"
      options={deptChoices}
      bind:value={filters.department}
    />
    <Select
      label="Course level"
      variant={wide.current ? "compact" : "field"}
      class="min-w-0 sm:w-44"
      options={levelChoices}
      bind:value={filters.level}
    />
    <div class="flex items-center justify-end gap-3 sm:ml-auto">
      <p class="text-label-md text-muted max-sm:sr-only" aria-live="polite">
        {shown.length} of {taking.length} {taking.length === 1 ? "course" : "courses"}
      </p>
      <ViewToggle pageKey="profile-taking" bind:value={view} />
    </div>
  </div>
{/snippet}

{#if personal.mode === "loading"}
  <div class="flex justify-center py-16"><Spinner size="lg" label="Loading your record" /></div>
{:else}
<div class="flex flex-col gap-6" aria-label={title} role="region">
  {#if taking.length > 0}
    <div class="min-w-0">
      <p class="text-heading-6 text-strong">{semester}</p>
      <p class="text-body-2 text-muted">
        {taking.length}
        {taking.length === 1 ? "course" : "courses"}, {fmt(units)} units{#if schedule !== null}. Sections from
          <a href={schedulesPath(termId, schedule.id)} class="text-accent-ink hover:underline">{schedule.name}</a
          >{/if}
      </p>
    </div>
  {/if}

  {#if missing.length > 0}
    <section
      aria-labelledby="taking-missing"
      class="flex flex-col gap-3 rounded-card border border-pressed bg-surface p-5 md:flex-row md:items-center"
    >
      <div class="min-w-0 flex-1">
        <h2 id="taking-missing" class="text-heading-6 text-strong">
          {missing.length === 1 ? "A course" : `${missing.length} courses`} in your schedule {missing.length === 1
            ? "isn't"
            : "aren't"} on your record
        </h2>
        <p class="mt-0.5 text-body-2 text-ink">
          {#each missing as id, i (id)}{i > 0 ? ", " : ""}<a
              href={coursePath(id, termId)}
              class="tabular font-semibold text-strong hover:text-accent-ink hover:underline">{codeOf(id)}</a
            >{/each}
          from {schedule?.name}. Add {missing.length === 1 ? "it" : "them"} as {missing.length === 1
            ? "a course"
            : "courses"} you're taking in {semester}.
        </p>
      </div>
      <Button variant="primary" icon={Plus} onclick={addMissing}>Add {missing.length === 1 ? "it" : "these"} to your record</Button>
    </section>
  {/if}

  {#if taking.length === 0}
    <EmptyState
      icon={Calendar1}
      title="Nothing in progress for {semester}"
      description="Courses on your record for {semester} show up here, with their sections and times from your schedule. Import your SIO record or add them under Courses Taken."
    >
      <SioImportButton variant="primary" />
      <Button href={profilePath("taken")}>Open Courses Taken</Button>
    </EmptyState>
  {:else if shown.length === 0}
    <div class="flex flex-wrap items-center gap-3 rounded-card border border-dashed border-line-strong px-4 py-5">
      <p class="text-body-2 text-muted">None of this semester's courses match these filters.</p>
      <Button size="sm" variant="secondary" onclick={() => (filters = { ...ALL_FILTERS })}>Clear filters</Button>
    </div>
  {:else if view === "cards"}
    <ul class="grid grid-cols-1 gap-4 lg:grid-cols-2">
      {#each shown as entry (entry.id)}
        <li class="flex *:flex-1">
          <TakingCard
            {entry}
            {data}
            scheduled={entry.courseId === null ? undefined : byCourse.get(entry.courseId)}
            hasSchedule={scheduleView !== null && schedule !== null}
          />
        </li>
      {/each}
    </ul>
  {:else}
    <ul class="divide-y divide-line overflow-hidden rounded-card border border-pressed bg-surface">
      {#each shown as entry (entry.id)}
        <TakingCard
          {entry}
          {data}
          list
          scheduled={entry.courseId === null ? undefined : byCourse.get(entry.courseId)}
          hasSchedule={scheduleView !== null && schedule !== null}
        />
      {/each}
    </ul>
  {/if}

  {#if taking.length > 0 && schedule === null && built}
    <p class="flex items-center gap-2 text-body-2 text-muted">
      <CalendarPlus aria-hidden="true" class="size-4 shrink-0" />
      <span>
        Make a schedule for {semester} to see each course's sections and times here.
        <a href={schedulesPath(termId)} class="text-accent-ink hover:underline">Open schedules</a>
      </span>
    </p>
  {/if}
</div>
{/if}
