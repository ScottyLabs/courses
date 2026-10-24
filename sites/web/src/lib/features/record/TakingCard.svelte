<script lang="ts">
  // A course being taken this semester (Figma "Profile Page (Currently Taking)"), as the shared small course card:
  // code and title, department, the sections and meeting times chosen in the active schedule, units and FCE hours.
  // `list` is the compact row.
  import { CalendarDays, Clock, Coins } from "@lucide/svelte";
  import CourseCardSmall from "$lib/components/course/CourseCardSmall.svelte";
  import { formatHours } from "$lib/core/fce";
  import { partLabel, timesAndRoom } from "$lib/core/planner/format";
  import { departmentKey, departmentLabel } from "$lib/core/record/taken-list";
  import { coursePath } from "$lib/core/routes";
  import { entryUnits, type RecordEntry } from "$lib/core/state/record";
  import type { ResolvedEntry } from "$lib/core/state/schedule-view";
  import { session } from "$lib/data";
  import type { RecordData } from "./record-data.svelte";

  let {
    entry,
    data,
    scheduled,
    hasSchedule,
    list = false,
  }: {
    entry: RecordEntry;
    data: RecordData;
    /** The course in the active schedule, with its chosen sections; undefined when the schedule doesn't have it. */
    scheduled: ResolvedEntry | undefined;
    /** Whether there's an active schedule for the semester at all. */
    hasSchedule: boolean;
    list?: boolean;
  } = $props();

  const id = $derived(entry.courseId ?? "");
  const info = $derived(data.lookup(id));
  const facts = $derived(data.facts(id));
  const units = $derived(entryUnits(entry, scheduled?.course.units ?? info.units));
  const hours = $derived(data.hours(id));
  const href = $derived(coursePath(id, entry.term));
  const parts = $derived(
    (scheduled?.parts ?? []).map((section) => Object.assign({ label: partLabel(section) }, timesAndRoom(section, !session.signedIn))),
  );
  const sectionNote = $derived.by(() => {
    if (!hasSchedule) return null;
    if (scheduled === undefined) return "Not in your schedule";
    if (scheduled.section === null) return "No section chosen in your schedule";
    if (scheduled.needsChild) return "No recitation chosen in your schedule";
    return null;
  });
  const fmt = (n: number): string => (Number.isInteger(n) ? String(n) : n.toFixed(1));
</script>

{#snippet meetings()}
  {#if parts.length > 0}
    <ul class="flex flex-col gap-1">
      {#each parts as part (part.label)}
        {#if list}
          <li class="flex items-start gap-2 text-body-2 text-strong">
            <span class="w-14 shrink-0 text-muted">{part.label}</span>
            <span class="min-w-0">
              {part.times}{#if part.room}<span class="text-muted">{" \u00b7 "}{part.room}</span>{/if}
            </span>
          </li>
        {:else}
          <li class="flex items-start justify-between gap-3 text-body-2 text-strong">
            <span class="min-w-0">
              {part.times}{#if part.room}<span class="text-muted">{" \u00b7 "}{part.room}</span>{/if}
            </span>
            <span class="inline-flex shrink-0 items-center gap-1.5">
              <CalendarDays aria-hidden="true" class="size-4" />
              {part.label}
            </span>
          </li>
        {/if}
      {/each}
    </ul>
  {:else if sectionNote}
    <p class="text-body-2 text-muted">{sectionNote}</p>
  {/if}
{/snippet}

{#snippet stats()}
  <p class="flex flex-wrap items-center gap-x-3 gap-y-1 text-body-2 text-ink">
    <span class="inline-flex items-center gap-1.5">
      <Coins aria-hidden="true" class="size-4" />
      <span class="tabular">{fmt(units)} units</span>
    </span>
    {#if hours !== null}
      <span aria-hidden="true" class="h-4 w-px bg-line-strong"></span>
      <span class="inline-flex items-center gap-1.5">
        <Clock aria-hidden="true" class="size-4" />
        <span class="tabular">{formatHours(hours)} hrs/week</span>
      </span>
    {/if}
  </p>
{/snippet}

{#if list}
  <li class="grid grid-cols-1 gap-x-4 gap-y-1.5 px-4 py-3 md:grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)_auto] md:items-start">
    <p class="flex min-w-0 items-baseline gap-2">
      <a {href} class="tabular shrink-0 text-label-lg text-strong hover:underline">{info.code}</a>
      <span class="min-w-0 truncate text-body-2 text-ink">{entry.label ?? info.title ?? "Not in the catalog"}</span>
    </p>
    <div class="min-w-0">{@render meetings()}</div>
    <div class="md:text-right">{@render stats()}</div>
  </li>
{:else}
  <CourseCardSmall
    courseId={id}
    termId={entry.term ?? null}
    title={entry.label ?? info.title ?? "Not in the catalog"}
    {units}
    deptLabel={departmentLabel(departmentKey(id, facts))}
  >
    <div class="my-1">{@render meetings()}</div>
  </CourseCardSmall>
{/if}
