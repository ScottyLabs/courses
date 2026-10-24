<script lang="ts">
  // A course from the academic record as a read-only row of the detailed layout.
  import type { RecordEntryLike } from "$lib/core/plan";
  import { seasonsLabel, type Satisfies } from "$lib/core/plan-table";
  import { codeOf } from "$lib/core/requisite-format";
  import { planPage } from "./context";
  import SatisfiesCell from "./SatisfiesCell.svelte";

  let { entry, units, satisfies }: { entry: RecordEntryLike; units: number; satisfies: Satisfies | null } =
    $props();

  const page = planPage();
  const title = $derived(entry.courseId === null ? undefined : page.lookup(entry.courseId).title);
  const offered = $derived(entry.courseId === null ? null : seasonsLabel(page.catalog?.courses[entry.courseId]));
</script>

<tr class="align-top text-ink">
  <td class="sticky left-0 z-[1] bg-surface py-1.5 pr-2 pl-2 whitespace-nowrap">
    <span class="sr-only">From your academic record:</span>
    {#if entry.courseId !== null}
      <a
        href={page.href(entry.courseId, entry.term)}
        class="tabular pl-[18px] text-label-lg text-ink hover:text-accent-ink hover:underline">{codeOf(entry.courseId)}</a
      >
    {:else}
      <span class="pl-[18px] text-label-lg">Transfer</span>
    {/if}
  </td>
  <td class="max-w-0 py-1.5 pr-2">
    <span class="flex min-w-0 items-center gap-1.5">
      <span class="truncate text-muted">{entry.label ?? title ?? "Not in the catalog"}</span>
      {#if entry.grade}<span class="shrink-0 rounded-tag bg-paper px-1.5 text-label-md">{entry.grade}</span>{/if}
    </span>
  </td>
  <td class="tabular py-1.5 pr-2 text-right text-muted">{units}</td>
  <td class="py-1.5 pr-2 text-label-md font-normal"><SatisfiesCell {satisfies} freeElective={false} /></td>
  <td class="py-1.5 pr-2 text-label-md font-normal text-muted">{offered ?? "\u2013"}</td>
  <td class="py-1.5 pr-2 text-label-md font-normal text-muted">{entry.grade ? `Taken, ${entry.grade}` : "On your record"}</td>
  <td></td>
  <td></td>
</tr>
