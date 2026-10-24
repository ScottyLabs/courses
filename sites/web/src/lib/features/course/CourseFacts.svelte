<script lang="ts">
  import { KeyRound } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import Chip from "$lib/components/ui/Chip.svelte";
  import { instructorSlug, type Term } from "$lib/core/data";
  import { departmentName } from "$lib/core/departments";
  import { courseMini, unitsText } from "$lib/core/explorer/summary";
  import { PASS_FAIL_ONLY_HELP } from "$lib/core/planner/format";
  import { codeOf } from "$lib/core/requisite-format";
  import { coursesPath, instructorPath } from "$lib/core/routes";
  import { sameCourseTitle } from "$lib/core/substitutes";
  import type { Course } from "$lib/core/types";
  import { getCourseHref } from "./links";

  let {
    course,
    term,
    layout = "row",
  }: {
    course: Course;
    term: Term;
    /** row: side by side under the title (full page); list: label and value rows (detail panel). */
    layout?: "row" | "list";
  } = $props();

  const courseHref = getCourseHref();
  const termId = $derived(term.meta.id);
  const slugByName = $derived(new Map(term.instructors.map((i) => [i.name, i.slug])));
  // Same title: the course under another number. Otherwise the sections only meet together (French I and II).
  const together = $derived(course.crossListed.filter((id) => term.courseById.has(id)));
  const crossListed = $derived(
    together.filter((id) => sameCourseTitle(course.title, term.courseById.get(id)?.title ?? "")),
  );
  const meetsWith = $derived(together.filter((id) => !crossListed.includes(id)));
  const mini = $derived(courseMini(course, term.meta));
  const listings = $derived([
    { label: "Also listed as", ids: crossListed },
    { label: "Meets with", ids: meetsWith },
  ]);
  const deptHref = $derived(`${coursesPath(termId)}&dept=${encodeURIComponent(course.dept)}`);
</script>

<!-- One fact: a label and its value, laid out for the chosen layout. -->
{#snippet fact(label: string, value: Snippet, wide = false)}
  {#if layout === "row"}
    <div class="min-w-0 {wide ? 'max-w-[26rem]' : ''}">
      <dt class="text-label-md text-muted">{label}</dt>
      <dd class="mt-0.5">{@render value()}</dd>
    </div>
  {:else}
    <div class="contents">
      <dt class="text-muted">{label}</dt>
      <dd class="min-w-0">{@render value()}</dd>
    </div>
  {/if}
{/snippet}

{#snippet units()}
  {unitsText(course.units)}{#if course.passFail}<span class="text-muted">, not factorable</span>{/if}
{/snippet}
{#snippet grading()}
  <Chip tone="neutral" size="sm">Pass/fail only</Chip>
  <span class="mt-0.5 block text-label-md text-muted">{PASS_FAIL_ONLY_HELP}</span>
{/snippet}
{#snippet department()}
  <a href={deptHref} class="underline decoration-line-strong underline-offset-2 hover:text-accent-ink"
    >{departmentName(course.dept)}</a
  >
  <span class="text-muted">({course.dept})</span>
{/snippet}
{#snippet level()}
  {course.level === "G" ? "Graduate" : "Undergraduate"}
{/snippet}
{#snippet session()}
  {mini}
{/snippet}
{#snippet registration()}
  <span class="inline-flex items-center gap-1.5">
    <KeyRound aria-hidden="true" class="size-3.5 text-muted" />Special permission required
  </span>
{/snippet}
{#snippet instructors()}
  <span class="flex flex-wrap gap-x-3 gap-y-0.5">
    {#each course.instructors as name (name)}
      <a
        href={instructorPath(slugByName.get(name) ?? instructorSlug(name), termId)}
        class="underline decoration-line-strong underline-offset-2 hover:text-accent-ink">{name}</a
      >
    {/each}
  </span>
{/snippet}

<dl
  class={layout === "row"
    ? "flex flex-wrap gap-x-10 gap-y-4 text-body-2 text-ink"
    : "grid grid-cols-[7rem_minmax(0,1fr)] gap-x-3 gap-y-1.5 text-body-2 text-ink"}
>
  {@render fact("Units", units)}
  {#if course.passFail}{@render fact("Grading", grading, true)}{/if}
  {@render fact("Department", department)}
  {@render fact("Level", level)}
  {#if mini}{@render fact("Session", session)}{/if}
  {#if course.permissionRequired}{@render fact("Registration", registration)}{/if}
  {#if course.instructors.length > 0}
    {@render fact(course.instructors.length === 1 ? "Instructor" : "Instructors", instructors, true)}
  {/if}
  {#each listings as { label, ids } (label)}
    {#if ids.length > 0}
      {#snippet links()}
        <span class="flex flex-wrap gap-x-3">
          {#each ids as id (id)}
            <a
              href={courseHref(id, termId)}
              title={term.courseById.get(id)?.title}
              class="tabular underline decoration-line-strong underline-offset-2 hover:text-accent-ink">{codeOf(id)}</a
            >
          {/each}
        </span>
      {/snippet}
      {@render fact(label, links, true)}
    {/if}
  {/each}
</dl>
