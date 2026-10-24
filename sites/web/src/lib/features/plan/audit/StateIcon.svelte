<script lang="ts">
  // A requirement's state as an icon: done, planned, in progress, still needed, advisor check, not needed.
  import { Circle, CircleCheck, CircleDashed, CircleHelp, Clock } from "@lucide/svelte";
  import type { DisplayState } from "$lib/core/degree/audit/format";

  let { state, class: className = "" }: { state: DisplayState; class?: string } = $props();

  const ICONS = {
    complete: CircleCheck,
    "complete-with-plan": Clock,
    "in-progress": CircleDashed,
    missing: Circle,
    manual: CircleHelp,
    skipped: Circle,
  };
  const COLORS: Record<DisplayState, string> = {
    complete: "text-ok",
    "complete-with-plan": "text-warn",
    "in-progress": "text-ink",
    missing: "text-danger",
    manual: "text-muted",
    skipped: "text-faint",
  };
  const Icon = $derived(ICONS[state]);
</script>

<Icon aria-hidden="true" class="shrink-0 {COLORS[state]} {className}" />
