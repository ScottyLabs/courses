<script lang="ts">
  // A requisite check's level as its icon and color (core/record/check-summary.ts's LEVEL_STYLE): a check when met,
  // a clock when planned, an alert when missing, an info mark when it can't be told.
  import { Check, Clock, Info, TriangleAlert } from "@lucide/svelte";
  import { LEVEL_STYLE } from "$lib/core/record/check-summary";
  import type { CheckLevel } from "$lib/core/requisites";

  let { level, class: className = "" }: { level: CheckLevel | "none"; class?: string } = $props();

  const ICONS = { check: Check, clock: Clock, info: Info, "triangle-alert": TriangleAlert };
  const style = $derived(LEVEL_STYLE[level]);
  const Icon = $derived(ICONS[style.icon]);
</script>

<Icon aria-hidden="true" class="shrink-0 {style.color} {className}" />
