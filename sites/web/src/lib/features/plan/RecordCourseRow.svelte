<script lang="ts">
  // A course from the academic record in its semester: read-only, with its grade.
  import type { RecordEntryLike } from "$lib/core/plan";
  import { codeOf } from "$lib/core/requisite-format";
  import { planPage } from "./context";

  let { entry, units }: { entry: RecordEntryLike; units: number } = $props();

  const page = planPage();
  const title = $derived(entry.courseId === null ? undefined : page.lookup(entry.courseId).title);
</script>

<li class="flex min-h-10 items-center gap-1.5 px-1 md:min-h-8">
  <span class="sr-only">From your academic record:</span>
  {#if page.dragEnabled}<span aria-hidden="true" class="w-3.5 shrink-0"></span>{/if}
  {#if entry.courseId !== null}
    <a
      href={page.href(entry.courseId, entry.term)}
      class="tabular shrink-0 text-label-lg text-ink hover:text-accent-ink hover:underline"
    >
      {codeOf(entry.courseId)}
    </a>
  {:else}
    <span class="shrink-0 text-label-lg text-ink">Transfer</span>
  {/if}
  <span class="min-w-0 flex-1 truncate text-body-2 text-muted">{entry.label ?? title ?? "Not in the catalog"}</span>
  {#if entry.grade}
    <span class="shrink-0 rounded-tag bg-paper px-1.5 text-label-md text-ink">{entry.grade}</span>
  {/if}
  <span class="tabular w-9 shrink-0 text-right text-label-md font-normal text-muted">{units}u</span>
  <span aria-hidden="true" class="w-8 shrink-0 max-md:w-10"></span>
</li>
