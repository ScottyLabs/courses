<!--
  Expected weekly workload from FCE hours, next to what the units suggest (Courseweave's ScheduleWorkload). Courses
  without FCE hours count at what their units suggest. With minis, each half of the semester is shown and the
  heavier one marked. FCEs are member data: signed out, a sign-in prompt takes its place.
-->
<script lang="ts">
  import MemberSignIn from "$lib/components/course/MemberSignIn.svelte";
  import WorkloadMeter, { TONE_TEXT } from "$lib/components/course/WorkloadMeter.svelte";
  import { catalog, resource } from "$lib/data";
  import type { Term } from "$lib/core/data";
  import {
    FCE_HOURS_HELP,
    WORKLOAD_TONE_LABEL,
    formatHours,
    workloadRatio,
    workloadTone,
  } from "$lib/core/fce";
  import { scheduleLoad } from "$lib/core/fce/workload";
  import type { ResolvedEntry } from "$lib/core/state/schedule-view";

  let { entries, term }: { entries: readonly ResolvedEntry[]; term: Term } = $props();

  const fce = resource(() => catalog.fceSummary());
  const visible = $derived(entries.filter((e) => !e.entry.hidden));
  const load = $derived(
    fce.value === undefined || fce.value === null
      ? null
      : scheduleLoad([...entries], term.meta.minis, fce.value.courses),
  );
  const peak = $derived(load?.halves && load.peak !== null ? load.halves[load.peak] : null);
  const tone = $derived(
    load === null
      ? null
      : peak
        ? workloadTone(peak.load.expected > 0 ? peak.load.hours / peak.load.expected : null)
        : workloadTone(workloadRatio(load.total, load.units)),
  );
  const missing = $derived(load?.rows.filter((r) => r.hours === null).length ?? 0);
</script>

{#if visible.length === 0}
  <p class="text-body-2 text-muted">Add courses to see how many hours a week they take.</p>
{:else if fce.status === "signin" || fce.status === "forbidden"}
  <MemberSignIn subject="expected hours from course evaluations" reason={fce.status} />
{:else if fce.status === "error"}
  <p class="text-body-2 text-muted">Couldn't load course evaluations. {fce.error}</p>
{:else if load === null}
  <p class="text-body-2 text-muted" aria-live="polite">
    {fce.status === "loading" ? "Loading course evaluations..." : "This catalog has no course evaluations."}
  </p>
{:else}
  <section aria-label="Expected workload" class="min-w-0">
    <p class="flex flex-wrap items-baseline gap-x-1.5 text-body-2 text-muted" title={FCE_HOURS_HELP}>
      <span class="tabular text-heading-4 text-strong">{formatHours(peak ? peak.load.hours : load.total)}</span>
      hrs/week{peak ? " at the busiest point" : ""} for {load.units} units
    </p>
    {#if tone}
      <p class="mt-0.5 text-label-md {TONE_TEXT[tone]}">{WORKLOAD_TONE_LABEL[tone].replace("its units", "your units")}</p>
    {/if}

    {#if load.halves}
      <ul class="mt-3 grid grid-cols-2 gap-2" aria-label="Workload by half of the semester">
        {#each load.halves as h, i (h.mini.number)}
          {@const heavier = i === load.peak}
          <li
            class="rounded-tag border px-2.5 py-1.5 {heavier ? 'border-line-strong bg-surface-2' : 'border-line'}"
            title="{h.mini.name}: full-semester courses plus {h.mini.name} minis. Their units suggest about {formatHours(
              h.load.expected,
            )} hours a week, counting mini units twice since minis run half the semester."
          >
            <span class="flex items-baseline justify-between gap-2 text-label-md">
              <span class={heavier ? "font-semibold text-strong" : "text-ink"}>{h.mini.name}</span>
              {#if heavier}<span class="text-label-sm font-semibold text-ink">Heavier</span>{/if}
            </span>
            <span class="tabular block text-body-2">
              <span class="font-semibold text-strong">{formatHours(h.load.hours)}</span>
              <span class="text-muted">hrs/wk, units suggest {formatHours(h.load.expected)}</span>
            </span>
          </li>
        {/each}
      </ul>
    {/if}

    <ul class="mt-3 flex flex-col gap-2">
      {#each load.rows as row (row.entry.course.id)}
        {@const mini = row.mini !== null}
        <li class="grid grid-cols-[4.25rem_minmax(0,1fr)_4.75rem] items-center gap-2 text-label-md">
          <span class="tabular font-semibold text-strong">{row.entry.course.code}</span>
          {#if row.hours !== null}
            <WorkloadMeter hours={row.hours} units={row.entry.course.units} {mini} />
          {:else}
            <span class="font-normal text-muted">No FCE hours</span>
          {/if}
          <span class="tabular text-right font-normal text-ink">
            {row.hours === null ? "-" : formatHours(row.hours)}<span class="text-muted"
              >/{row.entry.course.units ?? "VAR"}{mini ? "u mini" : ""}</span
            >
          </span>
        </li>
      {/each}
    </ul>
    <p class="mt-3 text-label-md font-normal text-muted">
      FCE hours include class time.{load.halves
        ? " Minis count only in their half, at twice their units a week."
        : ""}
      {missing > 0
        ? `${missing} ${missing === 1 ? "course has" : "courses have"} no FCE hours and ${missing === 1 ? "counts" : "count"} at what its units suggest.`
        : "Every course has FCE hours."}
    </p>
  </section>
{/if}
