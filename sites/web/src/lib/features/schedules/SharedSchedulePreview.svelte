<!--
  A schedule someone shared, read-only (Courseweave's SharedScheduleImport preview): its week, its courses and
  sections, and the courses this semester no longer offers.
-->
<script lang="ts">
  import WeekGrid from "$lib/components/calendar/WeekGrid.svelte";
  import { courseColorStyle } from "$lib/core/colors";
  import type { Term } from "$lib/core/data";
  import { meetingSummary } from "$lib/core/ics";
  import { partBlocks, partLabel } from "$lib/core/planner/format";
  import { codeOf } from "$lib/core/requisite-format";
  import { coursePath } from "$lib/core/routes";
  import { resolveEntry, unitTotals, type ResolvedEntry } from "$lib/core/state/schedule-view";
  import type { ScheduleEntry } from "$lib/core/state/store";
  import { tbaItems } from "./planner";
  import { buildScheduleView } from "$lib/core/state/schedule-view";

  let {
    entries,
    term,
    compact = false,
  }: {
    entries: readonly ScheduleEntry[];
    term: Term;
    /** A smaller calendar, for dialogs. */
    compact?: boolean;
  } = $props();

  const resolved = $derived.by(() => {
    const rows: ResolvedEntry[] = [];
    const skipped: string[] = [];
    for (const entry of entries) {
      const course = term.courseById.get(entry.courseId);
      if (course === undefined) skipped.push(codeOf(entry.courseId));
      else rows.push(resolveEntry(entry, course));
    }
    return { rows, skipped };
  });
  const view = $derived(
    buildScheduleView(
      { id: "shared", termId: term.meta.id, name: "", entries: [...entries], createdAt: 0, updatedAt: 0 },
      term,
    ),
  );
  const blocks = $derived(
    view.placed.flatMap((r) =>
      r.parts.flatMap((p) => partBlocks(r.course, p, r.entry.color, { conflict: view.conflicts.has(r.course.id) })),
    ),
  );
  const units = $derived(unitTotals(resolved.rows));
</script>

<div class="flex flex-col gap-4">
  <p class="text-body-2 text-muted">
    {term.meta.name} &middot; {resolved.rows.length}
    {resolved.rows.length === 1 ? "course" : "courses"} &middot; {units.total} units{units.nonFactorable > 0
      ? ` (${units.factorable} factorable)`
      : ""}
  </p>

  {#if blocks.length > 0}
    <div class="rounded-card border border-line bg-surface p-1.5 sm:p-2">
      <WeekGrid {blocks} {compact} tba={tbaItems(view, term.meta.id)} label="Shared schedule calendar" />
    </div>
  {/if}

  {#if resolved.rows.length > 0}
    <ul class="divide-y divide-line rounded-card border border-line bg-surface">
      {#each resolved.rows as { entry, course, section, parts, factorable } (course.id)}
        <li class="flex items-start gap-3 px-3 py-2.5" style={courseColorStyle(entry.color)}>
          <span aria-hidden="true" class="course-swatch mt-1 h-4 w-1 shrink-0 rounded-full"></span>
          <div class="min-w-0 flex-1 text-body-2">
            <p class="flex flex-wrap items-baseline gap-x-2">
              <a href={coursePath(course.code, term.meta.id)} class="tabular font-semibold text-strong hover:underline"
                >{course.code}</a
              >
              <span class="min-w-0 truncate text-ink">{course.title}</span>
              {#if entry.hidden}<span class="text-label-md font-normal text-muted">hidden</span>{/if}
              {#if !factorable}<span class="text-label-md font-normal text-muted">pass/fail</span>{/if}
            </p>
            <p class="text-muted">
              {section === null ? "No section chosen" : parts.map((p) => `${partLabel(p)}: ${meetingSummary(p)}`).join("; ")}
            </p>
          </div>
        </li>
      {/each}
    </ul>
  {:else}
    <p class="text-body-2 text-ink">None of the courses in this schedule are offered in {term.meta.name}.</p>
  {/if}

  {#if resolved.skipped.length > 0}
    <p class="text-body-2 text-warn">
      Not offered in {term.meta.name}, so left out: {resolved.skipped.join(", ")}
    </p>
  {/if}
</div>
