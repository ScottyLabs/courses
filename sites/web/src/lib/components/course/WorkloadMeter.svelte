<script lang="ts" module>
  import type { WorkloadTone } from "$lib/core/fce";

  /** Text color for a workload tone (Courseweave's WORKLOAD_TONE_CLASS, in this site's tokens). */
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
  }: {
    hours: number | null;
    units: number | null;
    /** Minis run half a semester, so their units suggest twice the weekly hours. */
    mini?: boolean;
    class?: string;
  } = $props();

  /** Scale ceiling, in hours a week. */
  const SCALE = 24;
  const fills: Record<WorkloadTone, string> = {
    light: "bg-ok",
    even: "bg-muted",
    heavy: "bg-warn",
    "very-heavy": "bg-danger",
  };
  const tone = $derived(workloadTone(workloadRatio(hours, units, mini)) ?? "even");
  const expected = $derived(expectedHours(units, mini));
  const pct = (v: number): string => `${Math.min(100, (v / SCALE) * 100)}%`;
</script>

<!--
  Reported hours as a bar, with a tick at the hours the units suggest (1 unit is about 1 hour a week over a full
  semester; 2 for a mini). The fill says whether the course runs lighter or heavier than that.
-->
{#if hours !== null}
  <span aria-hidden="true" class="relative block h-[5px] rounded-full bg-line {className}">
    <span class="absolute inset-y-0 left-0 rounded-full {fills[tone]}" style="width: {pct(hours)}"></span>
    {#if expected !== null && expected > 0}
      <span
        class="absolute -top-[3px] -bottom-[3px] w-[2px] rounded-full bg-strong"
        style="left: calc({pct(expected)} - 1px)"
      ></span>
    {/if}
  </span>
{/if}
