<script lang="ts">
  // What the requisite graph's colors, icons and lines mean, from the same looks the graph draws with.
  import { PILL_LOOKS, READY, type Look } from "./looks";

  let {
    unlocks,
    class: className = "",
  }: {
    /** The unlocks are shown, so "could take next" applies. */
    unlocks: boolean;
    class?: string;
  } = $props();

  const statuses = $derived<Look[]>([
    PILL_LOOKS.met,
    PILL_LOOKS["in-progress"],
    PILL_LOOKS.planned,
    PILL_LOOKS.missing,
    PILL_LOOKS.waived,
    PILL_LOOKS["grade-low"],
    ...(unlocks ? [READY] : []),
  ]);
</script>

<div class="flex min-w-0 flex-col gap-2 {className}">
  <h3 class="sr-only">Legend</h3>
  <ul class="flex flex-wrap gap-x-4 gap-y-2 text-label-md text-ink">
    {#each statuses as look (look.label)}
      <li class="flex items-center gap-1.5" title={look.help}>
        <span aria-hidden="true" class="grid size-5 place-items-center rounded-tag border {look.box}">
          <look.icon class="size-3.5 {look.tint}" strokeWidth={2.5} />
        </span>
        {look.label}<span class="sr-only">: {look.help}</span>
      </li>
    {/each}
    <li class="flex items-center gap-1.5">
      <span aria-hidden="true" class="grid h-5 w-6 place-items-center rounded-tag bg-hover px-1 text-label-md text-ink">C</span>
      Minimum grade
    </li>
    <li class="flex items-center gap-1.5">
      <svg aria-hidden="true" width="24" height="8" class="shrink-0"
        ><line x1="0" y1="4" x2="24" y2="4" class="stroke-ok" stroke-width="2" /></svg
      >
      Requirement met
    </li>
    <li class="flex items-center gap-1.5">
      <svg aria-hidden="true" width="24" height="8" class="shrink-0"
        ><line x1="0" y1="4" x2="24" y2="4" class="stroke-warn" stroke-width="2" /></svg
      >
      Met with planned courses
    </li>
    <li class="flex items-center gap-1.5">
      <svg aria-hidden="true" width="24" height="8" class="shrink-0"
        ><line x1="0" y1="4.5" x2="24" y2="4.5" class="stroke-line-strong" stroke-width="1" stroke-dasharray="4 3" /></svg
      >
      Corequisite: before or in the same semester
    </li>
  </ul>
</div>
