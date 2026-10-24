<script lang="ts">
  // The plan's overall progress as one bar: done, in progress (striped), planned and still needed. It fills only
  // when the plan reaches the unit total and meets every requirement. Courseweave's DegreeProgressBar.
  import { fmt } from "$lib/core/degree/audit/format";
  import type { DegreeProgress } from "$lib/core/degree/progress";

  let { progress, outside = 0 }: { progress: DegreeProgress; outside?: number } = $props();

  type Key = "done" | "inProgress" | "planned" | "needed";
  const FILL: Record<Key, string> = {
    done: "background: var(--color-ok)",
    inProgress:
      "background: repeating-linear-gradient(135deg, var(--color-ok) 0 3px, color-mix(in oklab, var(--color-ok) 50%, transparent) 3px 6px)",
    planned: "background: color-mix(in oklab, var(--color-ok) 38%, transparent)",
    needed: "background: var(--color-line)",
  };

  const withPlan = $derived(progress.done + progress.inProgress + progress.planned);
  const scale = $derived(Math.max(withPlan + progress.needed, 1e-9));
  const unitShort = $derived(
    progress.unitTarget === null ? 0 : Math.max(0, progress.unitTarget - withPlan),
  );
  // The unmet requirements decide how much is left (an estimate), or the unit total does (exact).
  const forRequirements = $derived(progress.requirementUnits > unitShort);
  const requirementsText = (n: number): string => `${n} unmet ${n === 1 ? "requirement" : "requirements"}`;
  const neededNote = $derived(
    forRequirements
      ? `for ${requirementsText(progress.unmet)}, estimated`
      : progress.unitTarget === null
        ? null
        : `to reach ${fmt(progress.unitTarget)} units${progress.unmet ? `, including ${requirementsText(progress.unmet)}` : ""}`,
  );
  const status = $derived(
    progress.needed > 0
      ? `${forRequirements ? "About " : ""}${fmt(progress.needed)} more ${progress.needed === 1 ? "unit" : "units"} to finish`
      : progress.inProgress + progress.planned > 0
        ? "Everything is planned"
        : "Complete",
  );
  const segments = $derived([
    { key: "done" as const, label: "Done", value: progress.done },
    { key: "inProgress" as const, label: "In progress", value: progress.inProgress },
    { key: "planned" as const, label: "Planned", value: progress.planned },
    { key: "needed" as const, label: "Still needed", value: progress.needed },
  ]);
  const shown = $derived(segments.filter((s) => s.value > 0 || s.key === "done"));
  const summary = $derived(shown.map((s) => `${fmt(s.value)} units ${s.label.toLowerCase()}`).join(", "));
</script>

<div class="flex flex-col gap-2">
  <div class="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1">
    <span class="text-label-md text-muted">Degree progress</span>
    <span class="text-label-lg text-strong" aria-live="polite">{status}</span>
  </div>
  <div role="img" aria-label="Degree progress: {summary}." class="flex h-3 gap-[2px] overflow-hidden rounded-full">
    {#each segments as s (s.key)}
      {#if s.value > 0}
        <div
          class="min-w-[3px]"
          style="{FILL[s.key]}; flex: {s.key === 'needed' ? '1 1 0' : `0 1 ${(s.value / scale) * 100}%`}"
        ></div>
      {/if}
    {/each}
  </div>
  <ul class="flex flex-wrap gap-x-4 gap-y-1 text-label-md font-normal text-ink">
    {#each shown as s (s.key)}
      <li class="flex items-center gap-1.5">
        <span aria-hidden="true" class="size-2.5 shrink-0 rounded-[3px]" style={FILL[s.key]}></span>
        {s.label}
        <span class="tabular font-semibold text-strong">{fmt(s.value)}</span>
        {#if s.key === "done" && progress.credit > 0}
          <span class="text-muted">({fmt(progress.credit)} from AP and transfer credit)</span>
        {/if}
        {#if s.key === "needed" && neededNote}<span class="text-muted">({neededNote})</span>{/if}
      </li>
    {/each}
    {#if outside > 0}
      <li class="text-muted">
        {fmt(outside)}
        {outside === 1 ? "unit" : "units"} in this plan {outside === 1 ? "doesn't" : "don't"} count toward the degree
      </li>
    {/if}
  </ul>
</div>
