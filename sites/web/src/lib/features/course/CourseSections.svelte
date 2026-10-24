<script lang="ts">
  import { ChevronDown, Info, LogIn } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { getSession } from "$lib/components/shell/session";
  import { partLabel, sectionCounts } from "$lib/core/planner/format";
  import { childSections, topSections } from "$lib/core/sections";
  import type { Course, TermMeta } from "$lib/core/types";
  import type { CourseSchedule } from "./course-schedule.svelte";
  import SectionRow from "./SectionRow.svelte";

  let { course, sched, meta }: { course: Course; sched: CourseSchedule; meta: TermMeta } = $props();

  /** Recitation lists longer than this can be folded away. */
  const COLLAPSE_AFTER = 6;

  const session = getSession();
  const tops = $derived(topSections(course));
  // Public term documents leave out instructors and rooms (decision 20).
  const redacted = $derived(session.status !== "signed-in");
  let collapsed = $state<string[]>([]);
  const notes = $derived(meta.notes ?? []);
</script>

<section aria-labelledby="course-sections" class="min-w-0">
  <div class="mb-2 flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
    <h2 id="course-sections" class="text-heading-4 text-strong">
      {course.sections.length === 1 ? "Section" : "Sections"}
    </h2>
    <p class="text-body-2 text-muted">{sectionCounts(course)} in {meta.name}</p>
  </div>
  <p class="mb-2 hidden text-label-md text-muted pointer-coarse:block">
    Tap a section to preview it in your schedule's calendar.
  </p>
  {#if notes.length > 0}
    <div class="mb-2 flex gap-1.5 text-label-md text-muted">
      <Info aria-hidden="true" class="mt-0.5 size-3.5 shrink-0" />
      <div>
        {#each notes as note (note)}<p>{note}</p>{/each}
      </div>
    </div>
  {/if}
  {#if redacted && session.status !== "loading"}
    <div
      class="mb-3 flex flex-wrap items-center gap-x-3 gap-y-2 rounded-card border border-dashed border-line-strong bg-surface-2 px-4 py-3"
    >
      <p class="min-w-0 flex-1 text-body-2 text-ink">Sign in to see instructors and rooms.</p>
      <Button size="sm" variant="secondary" icon={LogIn} onclick={() => session.signIn()}>Sign in</Button>
    </div>
  {/if}
  <div class="@container rounded-card border border-pressed bg-surface">
    <div
      aria-hidden="true"
      class="hidden border-b border-line px-2 py-2 text-label-md text-muted @lg:grid @lg:grid-cols-[3.75rem_minmax(0,1fr)_6rem] @lg:gap-x-2 @3xl:grid-cols-[4.5rem_20rem_minmax(0,1fr)_5rem_6rem] @3xl:gap-x-3"
    >
      <span>Section</span>
      <span class="grid grid-cols-[3rem_8.5rem_minmax(0,1fr)] gap-x-2">
        <span>Days</span><span>Time</span><span>Room</span>
      </span>
      <span class="hidden @3xl:block">Instructor</span>
      <span class="hidden text-right @3xl:block">Seats</span>
    </div>
    <ul aria-label="Sections of {course.code}">
      {#each tops as top (top.name)}
        {@const children = childSections(course, top.name)}
        {@const holdsChoice = sched.entry?.section?.name === top.name && sched.entry.child !== null}
        {@const open = holdsChoice || !collapsed.includes(top.name)}
        <li class="border-b border-line last:border-b-0">
          <SectionRow {course} section={top} {sched} {meta} {redacted} />
          {#if children.length > 0}
            <div class="pb-2 @3xl:pb-0">
              {#if children.length > COLLAPSE_AFTER && !holdsChoice}
                <button
                  type="button"
                  aria-expanded={open}
                  onclick={() =>
                    (collapsed = open ? [...collapsed, top.name] : collapsed.filter((name) => name !== top.name))}
                  class="ml-[3.75rem] flex min-h-10 items-center gap-1 rounded-tag px-2 text-label-md text-ink hover:text-strong @md:ml-[4.25rem] sm:min-h-8"
                >
                  <ChevronDown aria-hidden="true" class="size-3.5 transition-transform {open ? '' : '-rotate-90'}" />
                  {open ? "Hide" : "Show"}
                  {children.length} recitations for {partLabel(top)}
                </button>
              {/if}
              {#if open}
                <ul
                  aria-label="Recitations of {partLabel(top)}"
                  class="ml-3 border-l-2 border-line pl-1 @md:ml-6 @3xl:ml-0 @3xl:border-l-0 @3xl:pl-0"
                >
                  {#each children as child (child.name)}
                    <li class="border-t border-line first:border-t-0 @3xl:first:border-t">
                      <SectionRow {course} section={child} {sched} {meta} {redacted} />
                    </li>
                  {/each}
                </ul>
              {/if}
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  </div>
</section>
