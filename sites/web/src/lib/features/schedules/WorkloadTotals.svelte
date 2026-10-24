<!--
  The toolbar's "Total Workload" line (Figma; Courseweave's units footer): units, expected hours a week from FCEs,
  the factorable and pass/fail split, how many courses and conflicts. Hours are member data; signed out the line
  says so instead.
-->
<script lang="ts">
  import { Info, TriangleAlert } from "@lucide/svelte";
  import { catalog, resource, session } from "$lib/data";
  import type { Term } from "$lib/core/data";
  import { FACTORABLE_HELP, plural } from "$lib/core/planner/format";
  import { formatHours } from "$lib/core/fce";
  import { scheduleLoad } from "$lib/core/fce/workload";
  import { HEAVY_UNITS } from "$lib/core/plan";
  import type { ScheduleView } from "$lib/core/state/schedule-view";
  import { conflictCount } from "./planner";

  let { view, term }: { view: ScheduleView; term: Term } = $props();

  const id = $props.id();
  const fce = resource(() => catalog.fceSummary());
  const load = $derived(
    fce.value === undefined || fce.value === null
      ? null
      : scheduleLoad(view.entries, term.meta.minis, fce.value.courses),
  );
  const peak = $derived(load?.halves && load.peak !== null ? load.halves[load.peak] : null);
  const units = $derived(view.units);
  const conflicts = $derived(conflictCount(view));
  let help = $state(false);
</script>

<div class="mt-4 flex flex-wrap items-end justify-between gap-x-6 gap-y-2">
  <div class="flex flex-col gap-0.5">
    <p class="text-body-1 text-muted">Total Workload</p>
    <p class="flex flex-wrap items-center gap-x-2.5 gap-y-1 text-body-1 text-ink">
      <span class="tabular">
        {units.total}
        {units.hasVariable ? "Units plus variable" : units.total === 1 ? "Unit" : "Units"}
      </span>
      <span aria-hidden="true" class="h-5 w-px bg-line-strong"></span>
      {#if load !== null}
        <span class="tabular" title="Expected hours a week from course evaluations (FCEs), including class time">
          {formatHours(peak ? peak.load.hours : load.total)} hrs/week{peak ? " at peak" : ""}
        </span>
      {:else if fce.status === "signin"}
        <button type="button" class="text-accent-ink underline-offset-2 hover:underline" onclick={() => session.signIn()}>
          Sign in to see hrs/week
        </button>
      {:else if fce.status === "forbidden"}
        <span class="text-muted">Hours need a CMU account</span>
      {:else if fce.status === "error"}
        <span class="text-muted">Hours unavailable</span>
      {:else}
        <span class="text-muted">&hellip; hrs/week</span>
      {/if}
    </p>
    {#if view.entries.length > 0}
      <p class="flex flex-wrap items-center text-label-md font-normal text-muted">
        <span><strong class="font-semibold text-ink">{units.factorable}</strong> factorable</span>
        {#if units.nonFactorable > 0}
          <span>,&nbsp;<strong class="font-semibold text-ink">{units.nonFactorable}</strong> pass/fail</span>
        {/if}
        <button
          type="button"
          aria-expanded={help}
          aria-controls={help ? id : undefined}
          aria-label="What are factorable units?"
          title="What are factorable units?"
          onclick={() => (help = !help)}
          class="ml-0.5 inline-grid size-6 place-items-center rounded-full text-muted hover:bg-hover hover:text-ink"
        >
          <Info aria-hidden="true" class="size-3.5" />
        </button>
      </p>
    {/if}
    {#if help}
      <p {id} class="max-w-xl rounded-tag bg-surface-2 px-2.5 py-2 text-label-md font-normal text-ink">{FACTORABLE_HELP}</p>
    {/if}
    {#if units.factorable > HEAVY_UNITS}
      <p class="flex items-start gap-1 text-label-md font-normal text-warn">
        <TriangleAlert aria-hidden="true" class="mt-0.5 size-3.5 shrink-0" />
        Over {HEAVY_UNITS} factorable units, which usually needs your college's approval. Guidance, not policy.
      </p>
    {/if}
  </div>
  {#if view.entries.length > 0}
    <p class="text-right text-body-2" aria-live="polite">
      <span class="text-muted">{plural(view.entries.length, "course")}</span>
      {#if conflicts > 0}
        <span class="flex items-center justify-end gap-1 font-semibold text-danger">
          <TriangleAlert aria-hidden="true" class="size-3.5" />{plural(conflicts, "conflict")}
        </span>
      {:else}
        <span class="block text-ok">No conflicts</span>
      {/if}
    </p>
  {/if}
</div>
