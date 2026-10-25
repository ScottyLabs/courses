<!--
  What an instructor preview shows (decision 41): name and departments, what they teach in the link's semester (or,
  for all semesters, where and how much they teach), and their FCE aggregate in the window the "Aggregate FCEs"
  settings choose, as instructor cards show it. It reads the compact instructor document the list reads (decision 43)
  and the FCE rows the cards load, each once per session, so it never loads a semester's full document.
-->
<script lang="ts">
  import Skeleton from "../ui/Skeleton.svelte";
  import StarRating from "../ui/StarRating.svelte";
  import { aggregateFce, selectFceRows } from "../course/fce-aggregate";
  import { hoursText } from "../course/fce-format";
  import { fceOptions } from "../course/fce-options.svelte";
  import { departmentName } from "$lib/core/departments";
  import { FCE_HOURS_HELP, termName } from "$lib/core/fce";
  import { codeOf } from "$lib/core/requisite-format";
  import { termLabel } from "$lib/core/requisites";
  import { instructorPath } from "$lib/core/routes";
  import { catalog, resource } from "$lib/data";
  import { device, resolveTerm } from "$lib/state";
  import { instructorRows, instructorsIn } from "./data.svelte";
  import { findInstructor, teachingSummary } from "./directory";

  const COURSES_SHOWN = 4;

  let {
    slug,
    name = null,
    termId,
    id,
    room = false,
    openHref = $bindable(null),
  }: {
    /** The instructor page's URL slug. */
    slug: string;
    /** Their name as schedules give it, when known; it finds them before the slug does. */
    name?: string | null;
    /** A semester, "all", or null for the one the instructor page would pick. */
    termId: string | null;
    /** DOM id of the text, for aria-describedby. */
    id?: string;
    /** Leaves room beside the heading for a close button in the card's corner. */
    room?: boolean;
    /**
     * Set to their all-semesters page when the card found them only there (a link without a semester, to someone
     * not teaching in the one it would open), for "Open instructor" to go where the card's details come from.
     */
    openHref?: string | null;
  } = $props();

  const target = $derived({ slug, name });
  const terms = resource(() => catalog.terms());
  const all = $derived(termId === "all");
  const semester = $derived(
    all || terms.value === undefined ? null : resolveTerm(termId, terms.value, device.lastTerm),
  );
  const semesterName = $derived(
    semester === null ? "" : (terms.value?.find((t) => t.id === semester)?.name ?? termLabel(semester)),
  );

  const primary = $derived(all ? instructorsIn("all") : semester === null ? null : instructorsIn(semester));
  const here = $derived(primary?.value ? findInstructor(primary.value, target) : undefined);
  // Not teaching in the semester: who they are, from everyone in any semester or in course evaluations.
  const elsewhere = $derived(
    !all && primary?.status === "ready" && primary.value !== null && here === undefined ? instructorsIn("all") : null,
  );
  const found = $derived(elsewhere?.value ? findInstructor(elsewhere.value, target) : undefined);
  const inst = $derived(here ?? found);
  $effect(() => {
    openHref = here === undefined && found !== undefined ? instructorPath(found.slug, "all") : null;
  });
  const titles = $derived((here === undefined ? elsewhere?.value?.titles : primary?.value?.titles) ?? {});
  const loading = $derived(
    terms.status === "loading" ||
      (primary !== null && primary.status === "loading") ||
      (elsewhere !== null && elsewhere.status === "loading"),
  );
  const problem = $derived(primary?.status === "ready" ? elsewhere?.status : primary?.status);

  const courses = $derived(inst ? inst.courseIds.toSorted() : []);
  const counts = $derived.by((): string => {
    if (!inst) return "";
    const text = `${courses.length} ${courses.length === 1 ? "course" : "courses"}`;
    // People only course evaluations know teach no sections here; their courses are the ones evaluated.
    if (inst.sectionCount === 0) return text;
    return `${text}, ${inst.sectionCount} ${inst.sectionCount === 1 ? "section" : "sections"}`;
  });
  // In a semester: what they teach in it. Otherwise their semesters, from all semesters' document.
  const where = $derived(here !== undefined && !all ? semesterName : teachingSummary(inst?.terms ?? []));

  const rows = $derived(inst?.fceCourses ? instructorRows(inst.name, inst.fceCourses) : null);
  const windowed = $derived.by(() => {
    if (!rows?.value) return null;
    const chosen = selectFceRows(rows.value, fceOptions.value);
    // None in the chosen seasons (someone who teaches only in summer): their latest in any season, named below.
    return chosen.length > 0 ? chosen : selectFceRows(rows.value, { ...fceOptions.value, seasons: ["S", "M", "F"] });
  });
  const stats = $derived(windowed ? aggregateFce(windowed) : null);
  /** The semesters the figures cover: "Summer 2025", "Fall 2024 to Spring 2026". */
  const span = $derived.by((): string => {
    const covered = stats?.terms ?? [];
    const [newest] = covered;
    const oldest = covered.at(-1);
    if (newest === undefined || oldest === undefined) return "";
    return newest === oldest ? termName(newest) : `${termName(oldest)} to ${termName(newest)}`;
  });
  const ratings = $derived([
    { label: "Teaching rate", value: stats?.teachingRate ?? null },
    { label: "Course rate", value: stats?.courseRate ?? null },
  ]);
  const fallbackName = $derived(name ?? slug.replaceAll("-", " "));
</script>

<div {id} class="flex flex-col gap-2.5 text-body-2 text-ink">
  {#if loading}
    <p class="text-muted">Loading {name ?? "the instructor"}&hellip;</p>
  {:else if problem === "signin"}
    <p class="text-muted">Sign in to see instructors.</p>
  {:else if problem === "forbidden"}
    <p class="text-muted">Instructors need a CMU account.</p>
  {:else if problem === "error"}
    <p class="text-danger">This instructor didn't load. Check your connection and try again.</p>
  {:else if inst === undefined}
    <div class="min-w-0 {room ? 'pr-10' : ''}">
      <p class="text-heading-5 text-strong">{fallbackName}</p>
      <p class="text-muted">
        {primary?.value === null
          ? "Open their page to see what they teach."
          : "Nobody by this name teaches in any semester here or appears in course evaluations."}
      </p>
    </div>
  {:else}
    <div class="min-w-0 {room ? 'pr-10' : ''}">
      <p class="text-heading-5 text-strong">{inst.name}</p>
      {#if inst.depts.length > 0}
        <p class="text-label-md text-muted">{inst.depts.map((code) => departmentName(code)).join(", ")}</p>
      {/if}
    </div>

    <div class="flex flex-col gap-1">
      {#if here === undefined && semester !== null}
        <p class="text-label-md text-muted">Not teaching in {semesterName}</p>
      {/if}
      <p class="text-label-md text-ink">
        {where}{#if courses.length > 0}<span class="text-muted">: {counts}</span>{/if}
      </p>
      {#if courses.length > 0}
        <ul class="flex flex-col gap-0.5">
          {#each courses.slice(0, COURSES_SHOWN) as course (course)}
            <li class="flex min-w-0 items-baseline gap-2">
              <span class="tabular shrink-0 font-medium text-strong">{codeOf(course)}</span>
              {#if titles[course]}<span class="truncate">{titles[course]}</span>{/if}
            </li>
          {/each}
          {#if courses.length > COURSES_SHOWN}
            <li class="text-muted">and {courses.length - COURSES_SHOWN} more</li>
          {/if}
        </ul>
      {/if}
    </div>

    <div class="flex flex-col gap-1.5 border-t border-line pt-2 text-label-md">
      {#if inst.fceCourses === null || inst.fceCourses.length === 0}
        <p class="text-muted">No course evaluations match this name.</p>
      {:else if rows?.status === "error"}
        <p class="text-danger">Course evaluations didn't load.</p>
      {:else if stats === null}
        <span class="sr-only">Loading course evaluations</span>
        {#each ["w-44", "w-40", "w-28", "w-48"] as width (width)}<Skeleton class="h-4 {width}" />{/each}
      {:else if stats.terms.length === 0}
        <p class="text-muted">No course evaluations with responses.</p>
      {:else}
        <dl class="grid grid-cols-[auto_minmax(0,1fr)] items-center gap-x-3 gap-y-1">
          {#each ratings as box (box.label)}
            <dt class="text-muted">{box.label}</dt>
            <dd class="flex">
              {#if box.value === null}
                <span aria-label="No responses">&ndash;</span>
              {:else}
                <StarRating value={box.value} size="sm" label={box.label} />
              {/if}
            </dd>
          {/each}
          <dt class="text-muted">Workload</dt>
          <dd title={FCE_HOURS_HELP}>
            {#if stats.hours === null}
              <span aria-label="No responses">&ndash;</span>
            {:else}
              <span class="tabular">{hoursText(stats.hours)}</span> hrs/wk
            {/if}
          </dd>
          <dt class="text-muted">Semesters</dt>
          <dd><span class="tabular">{stats.terms.length}</span> <span class="text-muted">({span})</span></dd>
        </dl>
      {/if}
    </div>
  {/if}
</div>
