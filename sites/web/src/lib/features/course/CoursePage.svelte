<script lang="ts">
  import { ArrowRight, ChevronLeft } from "@lucide/svelte";
  import { MediaQuery } from "svelte/reactivity";
  import { goto } from "$app/navigation";
  import CourseFcePanel from "$lib/components/course/CourseFcePanel.svelte";
  import { catalogIndex, courseDetails, offerings, termData } from "$lib/components/course/data.svelte";
  import FriendsTaking from "$lib/components/course/FriendsTaking.svelte";
  import RequiresAndUnlocks from "$lib/components/course/RequiresAndUnlocks.svelte";
  import SaveButton from "$lib/components/course/SaveButton.svelte";
  import PageContainer from "$lib/components/layout/PageContainer.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
  import { buildUnlocks } from "$lib/core/catalog";
  import { departmentName } from "$lib/core/departments";
  import { catalogTitle } from "$lib/core/explorer/unoffered";
  import { nearestTerm } from "$lib/core/offerings";
  import { codeOf } from "$lib/core/requisite-format";
  import { termLabel } from "$lib/core/requisites";
  import { coursePath, coursesPath } from "$lib/core/routes";
  import { catalog, resource } from "$lib/data";
  import { device, isAllSemesters, lastOrigin, planOrigin, resolveTerm } from "$lib/state";
  import CatalogFacts from "./CatalogFacts.svelte";
  import CountsToward from "./CountsToward.svelte";
  import CourseAbout from "./CourseAbout.svelte";
  import CourseFacts from "./CourseFacts.svelte";
  import CourseHead from "./CourseHead.svelte";
  import CourseSections from "./CourseSections.svelte";
  import CourseSyllabi from "./CourseSyllabi.svelte";
  import { CourseSchedule } from "./course-schedule.svelte";
  import NotOfferedHere from "./NotOfferedHere.svelte";
  import OfferingHistory from "./OfferingHistory.svelte";
  import { requisitesPath } from "./paths";
  import RequisiteStatus from "./RequisiteStatus.svelte";
  import ScheduleActions from "./ScheduleActions.svelte";
  import ScheduleCalendar from "./ScheduleCalendar.svelte";

  let {
    courseId,
    requestedTerm,
  }: {
    /** Five-digit id; "" when the address isn't a course number. */
    courseId: string;
    /** The `term` query parameter, if any. */
    requestedTerm: string | null;
  } = $props();

  const code = $derived(courseId === "" ? "" : codeOf(courseId));
  const terms = resource(() => catalog.terms());
  const all = $derived(isAllSemesters(requestedTerm));
  // The semester underneath: the one asked for, else the last one visited, else the newest.
  const resolved = $derived(terms.value ? resolveTerm(requestedTerm, terms.value, device.lastTerm) : null);
  const explicit = $derived(requestedTerm !== null && requestedTerm === resolved);
  const offeringsData = $derived(offerings());
  const offered = $derived(offeringsData.value?.byId.get(courseId)?.terms ?? []);

  /**
   * What to show: a semester's sections, or the all-semesters view. Without a `term`, a course the remembered
   * semester doesn't offer opens in the nearest semester that does, and one no semester offers in the catalog view.
   */
  const view = $derived.by((): { kind: "loading" } | { kind: "all" } | { kind: "term"; termId: string } => {
    if (resolved === null) return { kind: terms.status === "loading" ? "loading" : "all" };
    if (all) return { kind: "all" };
    if (explicit) return { kind: "term", termId: resolved };
    if (offeringsData.status === "loading") return { kind: "loading" };
    if (offeringsData.value === null || offeringsData.value === undefined || offered.includes(resolved))
      return { kind: "term", termId: resolved };
    if (offered.length === 0) return { kind: "all" };
    return { kind: "term", termId: nearestTerm(offered, resolved) ?? resolved };
  });
  const termId = $derived(view.kind === "term" ? view.termId : null);
  const term = $derived(termId === null ? null : termData(termId));
  const course = $derived(term?.value?.courseById.get(courseId) ?? null);
  const index = $derived(catalogIndex());
  const entry = $derived(index.value?.courses[courseId]);
  const offering = $derived(offeringsData.value?.byId.get(courseId));
  const details = $derived(courseDetails(courseId));
  const title = $derived(course?.title ?? offering?.title ?? catalogTitle(entry?.title));
  const units = $derived(course ? course.units : entry?.unitsMax === undefined ? (entry?.units ?? null) : null);
  // The semester requisites and requirements are checked for: the one shown, or the one underneath.
  const checkTerm = $derived(termId ?? resolved);
  // Loaded here, before the sections that check requisites against it ask for it.
  const checkLoad = $derived(checkTerm === null ? null : termData(checkTerm));

  const sched = new CourseSchedule(() => (course && term?.value ? { course, term: term.value } : null));
  // Wide screens keep the calendar in view beside the sections; narrower ones show it after them.
  const wide = new MediaQuery("(min-width: 1024px)");

  // Opened from the degree plan: back returns there; from the course list, to the list as it was.
  const fromPlan = planOrigin();
  const origin = lastOrigin();
  const back = $derived.by(() => {
    if (fromPlan !== null) return { href: fromPlan, label: "Back to plan" };
    if (origin !== null && (origin === "/" || origin.startsWith("/?"))) return { href: origin, label: "Back to courses" };
    return { href: coursesPath(termId ?? resolved), label: "All courses" };
  });

  const unlocks = $derived(index.value ? (buildUnlocks(index.value).get(courseId) ?? []) : []);
  const departments = $derived(term?.value?.departments ?? []);
  const titleOf = (id: string): string | undefined =>
    term?.value?.courseById.get(id)?.title ?? index.value?.courses[id]?.title;
  const chipHref = (id: string): string => coursePath(id, termId !== null && term?.value?.courseById.has(id) ? termId : null);
  const offeredHere = (id: string): boolean => term?.value?.courseById.has(id) ?? false;

  const termOptions = $derived([
    ...offered.map((t) => ({ value: t, label: termLabel(t) })),
    ...(termId !== null && !offered.includes(termId) ? [{ value: termId, label: termLabel(termId) }] : []),
    { value: "all", label: "All semesters" },
  ]);
  const nextTerm = $derived(resolved === null ? undefined : nearestTerm(offered, resolved));

  /** A white section card on the page background. */
  const card = "min-w-0 rounded-card border border-pressed bg-surface p-5 sm:p-6";

  let tree = $state<HTMLElement>();
  function showTree(): void {
    tree?.scrollIntoView({ behavior: "smooth", block: "start" });
  }
</script>

<CourseHead code={code || "Course"} {title} description={details.value?.description} />

<PageContainer width="wide">
  <div class="mb-4 flex flex-wrap items-center gap-x-3 gap-y-2">
    <a
      href={back.href}
      class="-ml-1 inline-flex min-h-10 items-center gap-1 rounded-full pr-2 text-label-lg text-muted hover:text-ink"
    >
      <ChevronLeft aria-hidden="true" class="size-5" />{back.label}
    </a>
    {#if courseId !== "" && (offered.length > 0 || termId !== null)}
      <Select
        variant="compact"
        label="Semester"
        class="ml-auto w-44"
        options={termOptions}
        value={view.kind === "all" ? "all" : (termId ?? "")}
        onchange={(event) => {
          const next = event.currentTarget.value;
          void goto(coursePath(courseId, next), { noScroll: true });
        }}
      />
    {/if}
  </div>

  {#if courseId === ""}
    <EmptyState
      title="That isn't a course number"
      description="Course numbers look like 15-122. Search for the course on the courses page."
    >
      <Button variant="primary" href={coursesPath(resolved)}>Search courses</Button>
    </EmptyState>
  {:else if view.kind === "loading" || (term !== null && term.status === "loading") || checkLoad?.status === "loading" || (view.kind === "all" && (index.status === "loading" || offeringsData.status === "loading"))}
    <div class="flex flex-col gap-4" aria-busy="true" aria-live="polite">
      <span class="sr-only">Loading {code}</span>
      <Skeleton class="h-14 w-40" />
      <Skeleton class="h-8 w-2/3" />
      <Skeleton variant="text" lines={4} />
    </div>
  {:else if term !== null && term.status === "error"}
    <EmptyState title="This semester didn't load" description="Check your connection, then reload the page.">
      <Button onclick={() => location.reload()}>Reload</Button>
    </EmptyState>
  {:else if view.kind === "term" && termId !== null && !course}
    <NotOfferedHere {courseId} {termId} terms={terms.value ?? []} />
  {:else if view.kind === "all" && !title}
    <EmptyState
      title="No course {code}"
      description="It isn't in the course catalog or any semester this site has. Check the number and try again."
    >
      <Button variant="primary" href={coursesPath(resolved)}>Search courses</Button>
    </EmptyState>
  {:else}
    <article class="flex flex-col gap-8">
      <!-- Actions sit level with the bottom of the title; requisite details open below them. -->
      <header class="grid gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(18rem,26rem)] lg:gap-x-12">
        <div class="min-w-0 lg:self-end">
          <div class="flex items-start gap-3">
            <div class="min-w-0 flex-1">
              <p class="text-[44px] leading-none font-semibold tracking-tight text-strong sm:text-[64px]">{code}</p>
              <h1 class="mt-2 max-w-[32ch] text-heading-3 text-strong sm:text-[32px] sm:leading-10">{title}</h1>
            </div>
            <SaveButton {courseId} />
          </div>
          {#if course && term?.value}
            <p class="mt-1 text-body-1 text-muted">{departmentName(course.dept)} &middot; {term.value.meta.name}</p>
            <div class="mt-6"><CourseFacts {course} term={term.value} /></div>
          {:else}
            <p class="mt-1 text-body-1 text-muted">All semesters</p>
            <div class="mt-6"><CatalogFacts {courseId} terms={terms.value ?? []} /></div>
          {/if}
        </div>
        <div class="flex min-w-0 flex-col gap-3 lg:self-end">
          {#if course && termId !== null}
            <ScheduleActions {sched} />
            <FriendsTaking {courseId} {termId} />
          {:else if nextTerm}
            <div>
              <Button variant="primary" trailingIcon={ArrowRight} href={coursePath(courseId, nextTerm)}>
                Sections in {termLabel(nextTerm)}
              </Button>
            </div>
          {/if}
          {#if checkTerm !== null}
            <RequisiteStatus {courseId} termId={checkTerm} />
          {/if}
        </div>
      </header>

      <div class="grid gap-x-8 gap-y-8 {course ? 'lg:grid-cols-[minmax(0,1fr)_minmax(20rem,24rem)]' : ''}">
        <div class="flex min-w-0 flex-col gap-8">
          {#if course && term?.value}
            <CourseSections {course} {sched} meta={term.value.meta} />
            {#if !wide.current}<ScheduleCalendar {sched} />{/if}
          {/if}
          <div class={card}><CourseAbout {courseId} termId={checkTerm} onshowtree={showTree} /></div>
          <div class={card}><CountsToward {courseId} termId={checkTerm} /></div>
          {#if course}
            <div class={card}><OfferingHistory {courseId} currentTerm={termId} terms={terms.value ?? []} /></div>
          {/if}
        </div>
        {#if course && wide.current}
          <aside class="min-w-0" aria-label="Your schedule">
            <ScheduleCalendar {sched} hourHeight={32} class="sticky top-[calc(var(--header-h)+1rem)]" />
          </aside>
        {/if}
      </div>

      <div class={card}>
        <CourseFcePanel
          {courseId}
          {units}
          {course}
          current={course?.instructors ?? []}
          currentLabel={term?.value ? `Teaching in ${term.value.meta.name}` : undefined}
          wide
        />
      </div>

      <div class={card}><CourseSyllabi {courseId} {code} /></div>

      <section bind:this={tree} aria-labelledby="course-requisites" class="scroll-mt-24">
        <div class="mb-3 flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
          <h2 id="course-requisites" class="text-heading-4 text-strong">Prerequisites and what it unlocks</h2>
          <a
            href={requisitesPath(courseId, termId ?? resolved)}
            class="inline-flex min-h-10 items-center text-label-lg text-accent-ink underline-offset-2 hover:underline sm:min-h-8"
          >
            Explore the whole chain
          </a>
        </div>
        {#if index.status === "loading"}
          <Skeleton class="h-40" />
        {:else if index.status === "error"}
          <p class="text-body-2 text-danger">The course catalog didn't load. Reload the page to try again.</p>
        {:else if index.value}
          <RequiresAndUnlocks
            id={courseId}
            index={index.value}
            {unlocks}
            href={chipHref}
            title={titleOf}
            offered={termId === null ? undefined : offeredHere}
            termName={term?.value?.meta.name}
            {departments}
            headingLevel={3}
          />
        {/if}
      </section>
    </article>
  {/if}
</PageContainer>
