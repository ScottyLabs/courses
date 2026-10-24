<script lang="ts">
  // Progress toward a target: solid for work done, lighter for planned work on top of it.
  import { fmt } from "$lib/core/degree/audit/format";

  let {
    have,
    planned,
    need,
    label,
    class: className = "",
  }: { have: number; planned: number; need: number; label: string; class?: string } = $props();

  const pct = (x: number): string => `${Math.max(0, Math.min(100, (x / Math.max(need, 1e-9)) * 100))}%`;
  const plannedVisible = $derived(Math.max(0, Math.min(planned, need - have)));
</script>

<div
  role="progressbar"
  aria-label={label}
  aria-valuemin={0}
  aria-valuemax={need}
  aria-valuenow={Math.min(need, have + planned)}
  aria-valuetext="{fmt(have)} done{planned ? `, ${fmt(planned)} planned` : ''}, of {fmt(need)}"
  class="relative h-1.5 overflow-hidden rounded-full bg-line {className}"
>
  <div class="absolute inset-y-0 left-0 bg-ok" style:width={pct(have)}></div>
  <div
    class="absolute inset-y-0 bg-[color-mix(in_oklab,var(--color-ok)_42%,transparent)]"
    style:left={pct(have)}
    style:width={pct(plannedVisible)}
  ></div>
</div>
