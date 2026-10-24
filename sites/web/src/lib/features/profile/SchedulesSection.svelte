<!--
  The profile's Schedules section (Figma "Profile Page (Schedule)"): every schedule as a card with its semester, units,
  expected hours a week and courses, newest semester first, filtered by semester. Each card opens the schedule.
-->
<script lang="ts">
  import { CalendarPlus, Clock, ExternalLink, Layers, Users } from "@lucide/svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import CourseChip from "$lib/components/ui/CourseChip.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
  import { catalog, resource } from "$lib/data";
  import type { Term } from "$lib/core/data";
  import { formatHours } from "$lib/core/fce";
  import { scheduleLoad } from "$lib/core/fce/workload";
  import { scheduleUnits } from "$lib/core/planner/format";
  import { codeOf } from "$lib/core/requisite-format";
  import { termLabel, termOrdinal } from "$lib/core/requisites";
  import { coursePath, schedulesPath } from "$lib/core/routes";
  import { resolveEntries } from "$lib/core/state/schedule-view";
  import { personal, schedules, type ScheduleItem } from "$lib/state";

  let { title }: { title: string } = $props();

  let semester = $state("");

  const all = $derived(
    Object.values(schedules.items).toSorted(
      (a, b) => (termOrdinal(b.termId) ?? 0) - (termOrdinal(a.termId) ?? 0) || b.updatedAt - a.updatedAt,
    ),
  );
  const termIds = $derived([...new Set(all.map((s) => s.termId))]);
  const shown = $derived(semester === "" ? all : all.filter((s) => s.termId === semester));

  // Each semester's courses, for units and hours; a semester the catalog no longer has is left out.
  const terms = resource(async () => {
    const results = await Promise.allSettled(termIds.map((id) => catalog.term(id)));
    const out = new Map<string, Term>();
    for (const r of results) if (r.status === "fulfilled") out.set(r.value.meta.id, r.value);
    return out;
  });
  const fce = resource(() => catalog.fceSummary());

  function hours(s: ScheduleItem, term: Term | undefined): string | null {
    const summary = fce.value;
    if (term === undefined || summary === undefined || summary === null) return null;
    const load = scheduleLoad(resolveEntries(s, term), term.meta.minis, summary.courses);
    const peak = load.halves && load.peak !== null ? load.halves[load.peak] : null;
    return formatHours(peak ? peak.load.hours : load.total);
  }
</script>

<div class="flex flex-col gap-6">
  {#if termIds.length > 1}
    <Select
      bind:value={semester}
      label="Semester"
      variant="compact"
      class="w-48"
      options={[{ value: "", label: "All semesters" }, ...termIds.map((id) => ({ value: id, label: termLabel(id) }))]}
    />
  {/if}

  {#if personal.mode === "loading"}
    <div class="grid gap-6 xl:grid-cols-2">
      <Skeleton variant="block" class="h-40" />
      <Skeleton variant="block" class="h-40" />
    </div>
  {:else if all.length === 0}
    <EmptyState
      icon={CalendarPlus}
      title="No schedules yet"
      description="Plan a semester on the schedules page: pick courses and sections, and they show up here."
    >
      <Button variant="primary" href={schedulesPath()}>Make a schedule</Button>
    </EmptyState>
  {:else}
    <ul class="grid gap-6 xl:grid-cols-2" aria-label={title}>
      {#each shown as s (s.id)}
        {@const term = terms.value?.get(s.termId)}
        {@const visible = s.entries.filter((e) => !e.hidden)}
        {@const perWeek = hours(s, term)}
        <li>
          <Card padding="none" class="relative flex h-full flex-col gap-3 px-5 py-6">
            <div class="flex items-start justify-between gap-3">
              <h3 class="min-w-0 text-heading-6 text-strong">
                <a href={schedulesPath(s.termId, s.id)} class="hover:underline">{s.name}</a>
              </h3>
              <a
                href={schedulesPath(s.termId, s.id)}
                aria-label="Open {s.name}"
                title="Open schedule"
                class="-mt-1 -mr-1 grid size-10 shrink-0 place-items-center rounded-full text-ink hover:bg-hover"
              >
                <ExternalLink aria-hidden="true" class="size-5" />
              </a>
            </div>
            <p class="flex flex-wrap items-center gap-x-3 gap-y-1 text-body-1 text-ink">
              <span class="text-muted">{term?.meta.name ?? termLabel(s.termId)}</span>
              {#if term}
                <span class="inline-flex items-center gap-1.5">
                  <Layers aria-hidden="true" class="size-5" />{scheduleUnits(s, term)} units
                </span>
              {:else if terms.status === "loading"}
                <span class="text-muted">&hellip;</span>
              {/if}
              {#if perWeek !== null}
                <span aria-hidden="true" class="h-5 w-px bg-line-strong"></span>
                <span class="inline-flex items-center gap-1.5">
                  <Clock aria-hidden="true" class="size-5" />{perWeek} hrs/week
                </span>
              {/if}
              {#if s.visibility === "friends"}
                <span class="inline-flex items-center gap-1 text-label-md text-muted" title="Friends can see it">
                  <Users aria-hidden="true" class="size-4" />Friends
                </span>
              {/if}
            </p>
            {#if visible.length > 0}
              <ul class="flex flex-wrap gap-2" aria-label="Courses in {s.name}">
                {#each visible as e (e.courseId)}
                  <li><CourseChip code={codeOf(e.courseId)} href={coursePath(e.courseId, s.termId)} /></li>
                {/each}
              </ul>
            {:else}
              <p class="text-body-2 text-muted">No courses yet.</p>
            {/if}
          </Card>
        </li>
      {/each}
    </ul>
  {/if}
</div>
