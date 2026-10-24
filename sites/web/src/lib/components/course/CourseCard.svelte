<script lang="ts">
  import { Building2, Calendar, ChevronDown, Clock, ExternalLink, LockKeyhole } from "@lucide/svelte";
  import { getSession } from "$lib/components/shell/session";
  import Button from "$lib/components/ui/Button.svelte";
  import Chip from "$lib/components/ui/Chip.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import { departmentName } from "$lib/core/departments";
  import type { Course } from "$lib/core/types";
  import { termLabel } from "$lib/core/requisites";
  import { coursePath, coursesPath } from "$lib/core/routes";
  import { catalogIndex, courseDetails, courseFce, fceSummary, offeredTerms } from "./data.svelte";
  import { aggregateFce, selectFceRows } from "./fce-aggregate";
  import { fceOptions } from "./fce-options.svelte";
  import FceAggregate from "./FceAggregate.svelte";
  import FceTable from "./FceTable.svelte";
  import FriendsTaking from "./FriendsTaking.svelte";
  import RequisiteSummary from "./RequisiteSummary.svelte";
  import SaveButton from "./SaveButton.svelte";

  let {
    course,
    termId,
    offered: offeredProp,
    href: hrefProp,
    expanded = $bindable(false),
    headingLevel = 2,
    class: className = "",
  }: {
    /** The course as a semester lists it, or a section-less course from the all-semesters view. */
    course: Course;
    /** The semester the card describes at first; its select switches to another semester offering the course. */
    termId: string;
    /** Semesters offering it, newest first; defaults to the offerings document plus `termId`. */
    offered?: readonly string[];
    /** Where the title links; defaults to the course page for the card's semester. */
    href?: string;
    /** Shows the full description and the FCE aggregate and table. */
    expanded?: boolean;
    headingLevel?: 2 | 3;
    class?: string;
  } = $props();

  const session = getSession();
  const id = $props.id();

  let chosenTerm = $state<string | null>(null);
  const cardTerm = $derived(chosenTerm ?? termId);
  const terms = $derived.by((): readonly string[] => {
    if (offeredProp !== undefined) return offeredProp;
    const list = offeredTerms(course.id) ?? [];
    return list.includes(termId) ? list : [termId, ...list];
  });
  const termOptions = $derived(terms.map((term) => ({ value: term, label: termLabel(term) })));

  const entry = $derived(catalogIndex().value?.courses[course.id]);
  const details = $derived(courseDetails(course.id));
  const description = $derived(details.value?.description);
  const website = $derived(details.value?.website);

  const units = $derived.by(() => {
    if (course.units !== null) return `${course.units} ${course.units === 1 ? "unit" : "units"}`;
    if (entry?.unitsMax !== undefined) return `${entry.units ?? 0}\u2013${entry.unitsMax} units`;
    return "Variable units";
  });

  const fce = $derived(fceSummary());
  const hours = $derived(fce.value?.courses.get(course.id)?.hours ?? null);

  const rows = $derived(expanded && session.status === "signed-in" ? courseFce(course.id) : null);
  const selected = $derived(rows?.value ? selectFceRows(rows.value, fceOptions.value) : []);
  const aggregate = $derived(aggregateFce(selected));

  const pageHref = $derived(coursePath(course.code, cardTerm));
  const href = $derived(hrefProp ?? pageHref);
  const deptHref = $derived(coursesPath(cardTerm) + `${cardTerm ? "&" : "?"}dept=${encodeURIComponent(course.dept)}`);
</script>

<article
  class="course-card rounded-card border border-pressed bg-surface px-4 py-5 text-ink sm:px-9 sm:py-[30px] {className}"
  aria-labelledby="{id}-title"
>
  <div class="head">
    <svelte:element this={`h${headingLevel}`} id="{id}-title" class="[grid-area:title] min-w-0 text-heading-5">
      <a
        {href}
        data-course-preview="off"
        class="rounded-tag hover:underline hover:decoration-line-strong hover:underline-offset-4"
      >
        <span class="tabular mr-1.5 font-bold sm:mr-2">{course.code}</span>{course.title}
      </a>
    </svelte:element>

    <p class="[grid-area:units] text-right text-heading-5 whitespace-nowrap @2xl:text-left">{units}</p>

    <SaveButton courseId={course.id} class="[grid-area:star] -mt-1.5 -mr-2 self-start justify-self-end" />

    <p class="[grid-area:dept] min-w-0 truncate text-heading-6 text-muted">
      {departmentName(course.dept || "")}
      {#if course.level === "G"}<span class="text-label-md">&middot; Graduate</span>{/if}
    </p>

    <div class="[grid-area:hours] flex items-center justify-end gap-2 text-heading-6 @2xl:justify-start">
      <Clock aria-hidden="true" class="hidden size-6 shrink-0 @2xl:block" />
      {#if fce.status === "loading"}
        <span class="inline-block h-4 w-24 animate-pulse rounded-tag bg-line" aria-hidden="true"></span>
      {:else if fce.status === "signin" || fce.status === "forbidden"}
        <button
          type="button"
          onclick={() => session.signIn()}
          class="inline-flex items-center gap-1 rounded-tag text-body-2 text-muted hover:text-ink"
          title="FCE hours are for CMU students. Sign in to see them."
        >
          <LockKeyhole aria-hidden="true" class="size-3.5" />Sign in for hours
        </button>
      {:else if hours === null}
        <span class="text-body-2 text-muted">No FCE hours</span>
      {:else}
        <span class="whitespace-nowrap text-muted @2xl:text-ink"
          ><span class="tabular">{hours.toFixed(1)}</span> hrs/week</span
        >
      {/if}
    </div>

    <div class="[grid-area:chips] flex flex-wrap items-center gap-2">
      {#if course.dept}
        <Chip tone="soft" icon={Building2} href={deptHref} title="All {departmentName(course.dept)} courses"
          >{course.dept} Dept</Chip
        >
      {/if}
      {#if website}
        <Chip tone="soft" icon={ExternalLink} href={website}>Course Website</Chip>
      {/if}
    </div>

    <div class="[grid-area:term] flex items-center gap-2">
      <Calendar aria-hidden="true" class="size-6 shrink-0" />
      {#if termOptions.length > 1}
        <Select
          variant="compact"
          label="Semester for {course.code}"
          options={termOptions}
          bind:value={() => cardTerm, (next) => (chosenTerm = next)}
          class="w-[9.5rem]"
        />
      {:else if termOptions.length === 1}
        <span class="text-body-2">{termOptions[0]?.label}</span>
      {:else}
        <span class="text-body-2 text-muted">Not offered recently</span>
      {/if}
    </div>

    <FriendsTaking courseId={course.id} termId={cardTerm} class="[grid-area:friends] self-start justify-self-end" />

    <dl class="[grid-area:reqs] flex min-w-0 flex-col gap-x-10 gap-y-1 @2xl:flex-row @2xl:flex-wrap">
      <div class="flex min-w-0 items-center gap-2">
        <dt class="shrink-0 text-body-1 font-semibold">Prerequisites:</dt>
        <dd class="min-w-0"><RequisiteSummary courseId={course.id} termId={cardTerm} kind="prereqs" /></dd>
      </div>
      <div class="flex min-w-0 items-center gap-2">
        <dt class="shrink-0 text-body-1 font-semibold">Corequisites:</dt>
        <dd class="min-w-0"><RequisiteSummary courseId={course.id} termId={cardTerm} kind="coreqs" /></dd>
      </div>
    </dl>
  </div>

  <div id="{id}-more" class="mt-3">
    {#if details.status === "loading"}
      <div class="flex flex-col gap-2" aria-hidden="true">
        <span class="h-4 w-full animate-pulse rounded-tag bg-line"></span>
        <span class="h-4 w-11/12 animate-pulse rounded-tag bg-line"></span>
        <span class="h-4 w-2/3 animate-pulse rounded-tag bg-line"></span>
      </div>
    {:else if description}
      <p class="text-body-1 text-ink {expanded ? '' : 'line-clamp-5 sm:line-clamp-4'}">{description}</p>
    {:else}
      <p class="text-body-2 text-muted">No description in the course catalog.</p>
    {/if}

    {#if expanded}
      <div class="mt-5 flex flex-col gap-3">
        {#if session.status !== "signed-in"}
          <div
            class="flex flex-wrap items-center gap-x-3 gap-y-2 rounded-card bg-surface-2 px-4 py-3 text-body-2 text-ink"
          >
            <LockKeyhole aria-hidden="true" class="size-4 shrink-0 text-muted" />
            <span class="min-w-0 flex-1">Sign in with your CMU account to see course evaluations.</span>
            <Button size="sm" variant="primary" onclick={() => session.signIn()}>Sign in</Button>
          </div>
        {:else if rows === null || rows.status === "loading"}
          <div class="h-24 animate-pulse rounded-card bg-surface-2" aria-label="Loading course evaluations"></div>
        {:else if rows.status === "error"}
          <p class="text-body-2 text-danger">Couldn't load course evaluations: {rows.error}</p>
        {:else if (rows.value ?? []).length === 0}
          <p class="rounded-card bg-surface-2 px-4 py-3 text-body-2 text-muted">
            No course evaluations for {course.code} yet.
          </p>
        {:else}
          <FceAggregate {aggregate} />
          <FceTable rows={selected} caption="Course evaluations for {course.code}" />
        {/if}
      </div>
    {/if}
  </div>

  <div class="mt-3 flex flex-wrap items-center justify-between gap-2">
    <button
      type="button"
      aria-expanded={expanded}
      aria-controls="{id}-more"
      onclick={() => (expanded = !expanded)}
      class="-ml-2 inline-flex h-10 items-center gap-2 rounded-full px-2 text-body-1 text-muted hover:bg-hover hover:text-ink"
    >
      <ChevronDown aria-hidden="true" class="size-5 transition-transform {expanded ? 'rotate-180' : ''}" />
      {expanded ? "Show less" : "Expand All"}
    </button>
    {#if expanded}
      <a href={pageHref} data-course-preview="off" class="text-body-2 text-accent-ink underline-offset-4 hover:underline">
        Sections, requisites and syllabi
      </a>
    {/if}
  </div>
</article>

<style>
  .course-card {
    container: course-card / inline-size;
  }

  .head {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto auto;
    grid-template-areas:
      "title units star"
      "title hours hours"
      "dept dept dept"
      "chips chips chips"
      "reqs reqs reqs"
      "term term friends";
    column-gap: 12px;
    row-gap: 6px;
    align-items: center;
  }

  @container course-card (min-width: 42rem) {
    .head {
      grid-template-columns: minmax(0, 1.75fr) minmax(12.5rem, 1fr) auto;
      grid-template-areas:
        "title units star"
        "dept hours star"
        "chips term friends"
        "reqs reqs friends";
      column-gap: 24px;
      row-gap: 4px;
    }
  }
</style>
