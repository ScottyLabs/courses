<!--
  Reported hours as a bar, with a tick at the hours the units suggest (Courseweave's WorkloadMeter): 1 unit is about
  1 hour a week over a full semester, 2 for a mini. The fill says whether the course runs lighter or heavier.
-->
<script lang="ts" module>
  import type { WorkloadTone } from "$lib/core/fce";

  /** Text color for a workload tone. */
  export const TONE_TEXT: Record<WorkloadTone, string> = {
    light: "text-ok",
    even: "text-ink",
    heavy: "text-warn",
    "very-heavy": "text-danger",
  };
</script>

<script lang="ts">
  import { expectedHours, workloadRatio, workloadTone } from "$lib/core/fce";

  let {
    hours,
    units,
    mini = false,
    class: className = "",
  }: { hours: number; units: number | null; mini?: boolean; class?: string } = $props();

  const SCALE = 24;
  const FILL: Record<WorkloadTone, string> = {
    light: "bg-ok",
    even: "bg-muted",
    heavy: "bg-warn",
    "very-heavy": "bg-danger",
  };
  const tone = $derived(workloadTone(workloadRatio(hours, units, mini)) ?? "even");
  const expected = $derived(expectedHours(units, mini));
  const pct = (v: number): string => `${Math.min(100, (v / SCALE) * 100)}%`;
</script>

<span aria-hidden="true" class="relative block h-[5px] rounded-full bg-line {className}">
  <span class="absolute inset-y-0 left-0 rounded-full {FILL[tone]}" style="width:{pct(hours)}"></span>
  {#if expected !== null && expected !== undefined && expected > 0}
    <span
      class="absolute -top-[3px] -bottom-[3px] w-[2px] rounded-full bg-strong"
      style="left:calc({pct(expected)} - 1px)"
    ></span>
  {/if}
</span>
