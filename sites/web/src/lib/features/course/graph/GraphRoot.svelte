<script lang="ts">
  // The course in the middle of the requisite graph: its code (with whether the student has it), title and the
  // verdict on its requisites in brief ("Prerequisites met", "1 missing"). Links to its course page.
  import { Info } from "@lucide/svelte";
  import RequisiteLevelIcon from "$lib/components/course/RequisiteLevelIcon.svelte";
  import { codeOf } from "$lib/core/requisite-format";
  import type { Rect } from "./geometry";
  import { PILL_LOOKS } from "./looks";
  import type { GraphSummary } from "./status";
  import type { PillStatus } from "./types";

  let {
    rect,
    courseId,
    title,
    href,
    own,
    summary,
    unchecked,
    tabindex,
  }: {
    rect: Rect;
    courseId: string;
    title: string | undefined;
    href: string;
    /** Whether the student has the course itself, and what that rests on. */
    own: { status: PillStatus; note: string } | null;
    summary: GraphSummary | null;
    /** Nothing on record to check against yet. */
    unchecked: boolean;
    tabindex: number;
  } = $props();

  const OWN_WORDS: Record<PillStatus, string> = {
    met: "Taken",
    "in-progress": "In progress",
    planned: "Planned",
    waived: "Placed out",
    missing: "",
    "grade-low": "",
  };
  const ownLook = $derived(own === null ? null : PILL_LOOKS[own.status]);
</script>

<a
  {href}
  class="absolute flex min-w-0 flex-col justify-center rounded-card border-2 border-accent bg-surface px-3.5 shadow-popover"
  style="left:{rect.x}px;top:{rect.y}px;width:{rect.w}px;height:{rect.h}px"
  data-graph-key="root"
  {tabindex}
>
  <span class="flex min-w-0 items-center gap-2">
    <span class="text-heading-5 font-semibold text-strong tabular">{codeOf(courseId)}</span>
    {#if own !== null && ownLook !== null}
      <span
        class="inline-flex h-5 min-w-0 items-center gap-1 rounded-tag border px-1.5 text-label-md text-ink {ownLook.box}"
        title={own.note}
        ><ownLook.icon aria-hidden="true" class="size-3 shrink-0 {ownLook.tint}" strokeWidth={3} /><span
          aria-hidden="true"
          class="truncate">{OWN_WORDS[own.status]}</span
        ><span class="sr-only">{own.note}</span></span
      >
    {/if}
  </span>
  <span class="truncate text-body-2 text-ink">{title ?? ""}</span>
  {#if summary !== null}
    <span
      class="mt-1 flex min-w-0 items-center gap-1.5 text-label-md {summary.level === 'problem' && !unchecked
        ? 'text-danger'
        : 'text-ink'}"
    >
      {#if unchecked && summary.level !== "none"}
        <Info aria-hidden="true" class="size-3.5 shrink-0 text-muted" /><span class="truncate">Not checked yet</span>
      {:else}
        <RequisiteLevelIcon level={summary.level} class="size-3.5" /><span class="truncate">{summary.short}</span>
      {/if}
    </span>
  {/if}
</a>
