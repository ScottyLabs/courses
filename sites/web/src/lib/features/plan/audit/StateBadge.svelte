<script lang="ts">
  // A program's overall state as a small pill: "Complete", "Complete with plan", "In progress", "Not started".
  import type { DisplayState } from "$lib/core/degree/audit/format";
  import StateIcon from "./StateIcon.svelte";

  let { state }: { state: DisplayState } = $props();

  const TONE: Record<DisplayState, string> = {
    complete: "bg-ok-bg text-ok",
    "complete-with-plan": "bg-warn-bg text-warn",
    "in-progress": "bg-paper text-ink",
    missing: "bg-danger-bg text-danger",
    manual: "bg-paper text-muted",
    skipped: "bg-paper text-muted",
  };
  const TEXT: Record<DisplayState, string> = {
    complete: "Complete",
    "complete-with-plan": "Complete with plan",
    "in-progress": "In progress",
    missing: "Not started",
    manual: "Advisor check",
    skipped: "Not needed",
  };
</script>

<span class="inline-flex h-6 shrink-0 items-center gap-1 rounded-full px-2 text-label-md {TONE[state]}">
  <StateIcon {state} class="size-3.5" />
  {TEXT[state]}
</span>
