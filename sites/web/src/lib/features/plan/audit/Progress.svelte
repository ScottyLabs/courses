<script lang="ts">
  // A requirement's progress bar with "9 of 27 units, 18 planned".
  import { progressText } from "$lib/core/degree/audit/format";
  import ProgressBar from "./ProgressBar.svelte";

  let {
    p,
    unit,
    name,
  }: { p: { have: number; planned: number; need: number }; unit: "units" | "courses"; name: string } =
    $props();
</script>

<div class="mt-1.5 flex max-w-[420px] flex-wrap items-center gap-x-2 gap-y-1">
  <ProgressBar have={p.have} planned={p.planned} need={p.need} label="{name} progress" class="min-w-24 flex-1" />
  <span class="tabular shrink-0 text-label-md text-muted">{progressText(p, unit)}</span>
</div>
