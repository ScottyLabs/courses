<script lang="ts">
  import { BookmarkPlus } from "@lucide/svelte";
  import { MediaQuery } from "svelte/reactivity";
  import { profileBand } from "$lib/components/layout/profile-band.svelte";
  import CourseCardSmall from "$lib/components/course/CourseCardSmall.svelte";
  import CourseList from "$lib/components/course/CourseList.svelte";
  import CourseRow from "$lib/components/course/CourseRow.svelte";
  import { catalogIndex, offerings } from "$lib/components/course/data.svelte";
  import SaveButton from "$lib/components/course/SaveButton.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
  import ViewToggle, { storedView, type ListView } from "$lib/components/ui/ViewToggle.svelte";
  import { departmentName } from "$lib/core/departments";
  import { codeOf } from "$lib/core/requisite-format";
  import { coursePath, coursesPath } from "$lib/core/routes";
  import type { Course } from "$lib/core/types";
  import { saved } from "$lib/state";

  let { title }: { title: string } = $props();

  let view = $state<ListView>(storedView("saved") ?? "cards");
  let department = $state("");
  let level = $state("");
  // Phones get the taller field selects (40px tap targets); wider screens the Figma's compact filter boxes.
  const wide = new MediaQuery("(min-width: 40rem)");

  // The filters, count and cards/list switch sit in the section's title band, as in the Figma.
  profileBand(() => ({ filters: saved.ids.length > 0 ? savedFilters : undefined }));

  const index = $derived(catalogIndex());
  const offered = $derived(offerings());
  const loading = $derived(index.status === "loading" || offered.status === "loading");

  /** The department of a course no built semester offers: the one other courses with its number prefix have. */
  const deptByPrefix = $derived.by(() => {
    const out = new Map<string, string>();
    for (const o of offered.value?.byId.values() ?? [])
      if (!out.has(o.id.slice(0, 2))) out.set(o.id.slice(0, 2), o.dept);
    return out;
  });

  /** Saved courses aren't tied to a semester: what the catalog and the offerings document say about each. */
  const courses = $derived.by(() =>
    saved.ids.toReversed().map((id): { course: Course; terms: readonly string[] } => {
      const offering = offered.value?.byId.get(id);
      const entry = index.value?.courses[id];
      const courseLevel = offering?.level === "G" || Number(id[2]) >= 6 ? "G" : "U";
      return {
        course: {
          id,
          code: codeOf(id),
          title: offering?.title ?? entry?.title ?? "",
          units: entry?.unitsMax === undefined ? (entry?.units ?? null) : null,
          dept: offering?.dept ?? deptByPrefix.get(id.slice(0, 2)) ?? "",
          level: courseLevel,
          instructors: [],
          sections: [],
          crossListed: [],
        },
        terms: offering?.terms ?? [],
      };
    }),
  );

  const departmentOptions = $derived(
    [...new Set(courses.map((c) => c.course.dept).filter((d) => d !== ""))]
      .map((code) => ({ value: code, label: departmentName(code) }))
      .toSorted((a, b) => a.label.localeCompare(b.label)),
  );
  const shown = $derived(
    courses.filter(
      ({ course }) => (department === "" || course.dept === department) && (level === "" || course.level === level),
    ),
  );
</script>

{#snippet savedFilters()}
  <div class="grid w-full grid-cols-2 gap-3 sm:flex sm:flex-wrap sm:items-center">
    <Select
      variant={wide.current ? "compact" : "field"}
      label="Department"
      options={[{ value: "", label: "Any department" }, ...departmentOptions]}
      bind:value={department}
      class="min-w-0 sm:w-[11.75rem]"
    />
    <Select
      variant={wide.current ? "compact" : "field"}
      label="Course level"
      options={[
        { value: "", label: "Any level" },
        { value: "U", label: "Undergraduate" },
        { value: "G", label: "Graduate" },
      ]}
      bind:value={level}
      class="min-w-0 sm:w-[11.75rem]"
    />
    <div class="col-span-2 flex items-center justify-between gap-3 sm:ml-auto">
      <p class="text-label-md text-muted" aria-live="polite">
        {shown.length === courses.length ? `${courses.length} saved` : `${shown.length} of ${courses.length} saved`}
      </p>
      <ViewToggle pageKey="saved" bind:value={view} />
    </div>
  </div>
{/snippet}

<h2 class="sr-only">{title}</h2>

{#if saved.ids.length === 0}
  <EmptyState
    icon={BookmarkPlus}
    title="No saved courses yet"
    description="Save courses with the star on the Courses page or a course page, and they're listed here."
  >
    <Button variant="primary" href={coursesPath()}>Browse courses</Button>
  </EmptyState>
{:else}
  {#if loading}
    <div class="grid gap-6 xl:grid-cols-2">
      {#each saved.ids.slice(0, 4) as id (id)}
        <div class="rounded-card border border-pressed bg-surface px-5 py-4">
          <Skeleton variant="text" class="w-3/4" />
          <Skeleton variant="text" class="mt-2 w-1/3" />
        </div>
      {/each}
    </div>
  {:else if shown.length === 0}
    <EmptyState
      size="sm"
      icon={BookmarkPlus}
      title="No saved courses match"
      description="Choose another department or level."
    />
  {:else if view === "cards"}
    <ul class="grid gap-6 xl:grid-cols-2" aria-label={title}>
      {#each shown as { course, terms } (course.id)}
        <li>
          <CourseCardSmall
            courseId={course.id}
            termId={terms[0] ?? null}
            title={course.title || undefined}
            units={course.units}
            dept={course.dept}
          >
            {#snippet actions()}
              <SaveButton courseId={course.id} size="sm" />
            {/snippet}
          </CourseCardSmall>
        </li>
      {/each}
    </ul>
  {:else}
    <CourseList label={title} allSemesters>
      {#each shown as { course, terms } (course.id)}
        <CourseRow {course} termId={null} offered={terms} href={coursePath(course.code, terms[0] ?? null)} />
      {/each}
    </CourseList>
  {/if}
{/if}
