<script lang="ts">
  // The detailed layout's Satisfies cell: the requirements a course counts toward, each opening its progress; or
  // "Free elective".
  import type { Satisfies, SatisfiesItem } from "$lib/core/plan-table";
  import type { RequirementState } from "$lib/core/plan-types";
  import Popover from "$lib/components/ui/Popover.svelte";

  let { satisfies, freeElective }: { satisfies: Satisfies | null; freeElective: boolean } = $props();

  const STATE_TEXT: Partial<Record<RequirementState, string>> = {
    complete: "Done",
    "complete-with-plan": "Done once planned courses are taken",
    "in-progress": "In progress",
    missing: "Not started",
  };
  const fmtNumber = (n: number): string => (Number.isInteger(n) ? String(n) : n.toFixed(1));

  function progressLine(item: SatisfiesItem): string | null {
    const p = item.units ?? item.courses;
    if (p === undefined) return null;
    const unit = item.units ? "units" : "courses";
    return `${fmtNumber(p.have)}${p.planned ? ` + ${fmtNumber(p.planned)} planned` : ""} of ${fmtNumber(p.need)} ${unit}`;
  }

  const linkClass =
    "min-h-10 text-left leading-snug underline decoration-line-strong decoration-dotted underline-offset-2 hover:text-strong md:min-h-0";
</script>

{#if satisfies && satisfies.items.length > 0}
  <span class="flex flex-col items-start gap-0.5">
    {#each satisfies.items as item (`${item.program}-${item.nodeId}`)}
      {@const line = progressLine(item)}
      <Popover label={item.label} placement="bottom-start" class="w-72 max-w-[calc(100vw-1rem)] p-3">
        {#snippet trigger(props)}
          <button type="button" {...props} class="{linkClass} text-ink">{item.label}</button>
        {/snippet}
        {#snippet children()}
          <p class="text-label-md font-normal text-muted">{item.program}</p>
          {#if item.path.length > 0}
            <p class="mt-0.5 text-label-md font-normal text-muted">{item.path.join(" \u203a ")} &rsaquo;</p>
          {/if}
          <p class="mt-0.5 text-label-lg text-strong">{item.label}</p>
          <p class="mt-1.5 text-body-2 text-ink">
            {STATE_TEXT[item.state] ?? ""}{#if line}<span class="text-muted"
                >{STATE_TEXT[item.state] ? ", " : ""}{line}</span
              >{/if}
          </p>
          {#if satisfies.items.length > 1}
            <p class="mt-1.5 text-label-md font-normal text-muted">
              This course counts toward {satisfies.items.length} requirements.
            </p>
          {/if}
        {/snippet}
      </Popover>
    {/each}
  </span>
{:else if freeElective}
  <Popover label="Free elective" placement="bottom-start" class="w-72 max-w-[calc(100vw-1rem)] p-3">
    {#snippet trigger(props)}
      <button type="button" {...props} class="{linkClass} text-muted">Free elective</button>
    {/snippet}
    {#snippet children()}
      <p class="text-body-2 text-ink">Doesn't count toward a specific requirement.</p>
      <p class="mt-1 text-label-md font-normal text-muted">Its units still count toward your degree's unit total.</p>
    {/snippet}
  </Popover>
{:else}
  <span class="text-muted">&ndash;</span>
{/if}
