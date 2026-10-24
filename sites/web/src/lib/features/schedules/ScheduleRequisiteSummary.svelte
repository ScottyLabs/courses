<!--
  The schedule's courses checked against the student's record (Courseweave's ScheduleRequisiteSummary): how many have
  unmet prerequisites, or something to double-check, with one line each. Courses taken before this semester, and
  courses planned or scheduled for earlier semesters, count; this schedule's own courses count as corequisites.
-->
<script lang="ts">
  import { Check, ChevronDown, Clock, Info, TriangleAlert } from "@lucide/svelte";
  import { catalogIndex } from "$lib/components/course/data.svelte";
  import { termRequisites } from "$lib/components/course/requisite-context.svelte";
  import type { Term } from "$lib/core/data";
  import { summarizeCheck } from "$lib/core/record/check-summary";
  import { checkCourse } from "$lib/core/requisites";
  import { coursePath, profilePath } from "$lib/core/routes";
  import type { ResolvedEntry } from "$lib/core/state/schedule-view";
  import { record } from "$lib/state";

  let { entries, term }: { entries: readonly ResolvedEntry[]; term: Term } = $props();

  const index = $derived(catalogIndex());
  // The same context the course pages check against, shared through the course components' cache.
  const ctx = $derived(termRequisites(term.meta.id).ctx);
  let open = $state(true);
  const id = $props.id();

  const rows = $derived.by(() => {
    const catalog = index.value;
    if (catalog === undefined || ctx === null) return null;
    const out: { code: string; courseId: string; text: string; level: string }[] = [];
    for (const { course } of entries) {
      const entry = catalog.courses[course.id];
      const check = checkCourse(course.id, entry, ctx);
      const summary = summarizeCheck(check, entry);
      if (summary.level === "problem" || check.issues.some((i) => i.severity === "warning"))
        out.push({ code: course.code, courseId: course.id, text: summary.text, level: summary.level });
    }
    return out;
  });
  const problems = $derived(rows?.filter((r) => r.level === "problem").length ?? 0);
</script>

{#if entries.length > 0}
  {#if index.status === "error"}
    <p class="text-body-2 text-muted">Couldn't check requisites. {index.error}</p>
  {:else if rows === null}
    <p class="text-body-2 text-muted" aria-live="polite">Checking requisites&hellip;</p>
  {:else if rows.length === 0}
    <p class="flex items-center gap-2 text-body-2 text-ink" aria-live="polite">
      <Check aria-hidden="true" class="size-4 shrink-0 text-ok" />
      {record.entries.length > 0 ? "Requisites look good for every course" : "No requisite problems found"}
    </p>
  {:else}
    <section class="rounded-field border border-line" aria-label="Requisite check">
      <h3>
        <button
          type="button"
          aria-expanded={open}
          aria-controls={id}
          onclick={() => (open = !open)}
          class="flex min-h-11 w-full items-center gap-2 px-3 text-left text-label-lg text-strong"
        >
          <TriangleAlert aria-hidden="true" class="size-4 shrink-0 {problems > 0 ? 'text-danger' : 'text-warn'}" />
          <span class="min-w-0 flex-1">
            {problems > 0
              ? `${problems} ${problems === 1 ? "course needs" : "courses need"} requisites`
              : `${rows.length} ${rows.length === 1 ? "course" : "courses"} to double-check`}
          </span>
          <ChevronDown aria-hidden="true" class="size-4 shrink-0 transition-transform {open ? 'rotate-180' : ''}" />
        </button>
      </h3>
      <div {id} hidden={!open} class="flex flex-col gap-3 border-t border-line px-3 pt-2 pb-3">
        <ul class="flex flex-col gap-2.5">
          {#each rows as row (row.courseId)}
            <li class="flex min-w-0 items-start gap-2">
              {#if row.level === "problem"}
                <TriangleAlert aria-hidden="true" class="mt-0.5 size-4 shrink-0 text-danger" />
              {:else if row.level === "planned"}
                <Clock aria-hidden="true" class="mt-0.5 size-4 shrink-0 text-warn" />
              {:else}
                <Info aria-hidden="true" class="mt-0.5 size-4 shrink-0 text-muted" />
              {/if}
              <span class="min-w-0 text-body-2">
                <a href={coursePath(row.code, term.meta.id)} class="tabular font-semibold text-strong hover:underline"
                  >{row.code}</a
                >
                <span class={row.level === "problem" ? "text-danger" : "text-ink"}>{row.text}</span>
              </span>
            </li>
          {/each}
        </ul>
        {#if record.entries.length === 0}
          <p class="text-label-md font-normal text-muted">
            Courses you've taken count toward prerequisites.
            <a href={profilePath("taken")} class="text-accent-ink underline underline-offset-2">Add them to your record</a>.
          </p>
        {/if}
      </div>
    </section>
  {/if}
{/if}
