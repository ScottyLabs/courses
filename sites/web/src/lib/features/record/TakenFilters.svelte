<!--
  Courses Taken's filters, in the profile's title band as the Figma's Courses Taken page has them: department, course
  level and semester, the count and the cards/list switch. They filter the courses taken list (TakenCourses).
-->
<script lang="ts">
  import { MediaQuery } from "svelte/reactivity";
  import Select from "$lib/components/ui/Select.svelte";
  import type { Option } from "$lib/components/ui/types";
  import ViewToggle, { storedView, type ListView } from "$lib/components/ui/ViewToggle.svelte";
  import { departmentOptions, levelOptions, matchesFilters, semesterOptions, takenEntries } from "$lib/core/record/taken-list";
  import { termLabel } from "$lib/core/requisites";
  import { personal, record } from "$lib/state";
  import { recordData } from "./record-data.svelte";
  import { takenList } from "./taken-list.svelte";

  const data = recordData();
  // Phones get the taller field selects (40px tap targets); wider screens the Figma's compact filter boxes.
  const wide = new MediaQuery("(min-width: 40rem)");

  const facts = (id: string): ReturnType<typeof data.facts> => data.facts(id);
  const taken = $derived(takenEntries(record.entries));
  const shown = $derived(taken.filter((e) => matchesFilters(e, takenList.filters, facts)).length);
  const all = (label: string): Option => ({ value: "", label });
  const deptChoices = $derived<Option[]>([all("All departments"), ...departmentOptions(taken, facts)]);
  const levelChoices = $derived<Option[]>([all("All levels"), ...levelOptions(taken, facts)]);
  const semesterChoices = $derived<Option[]>([all("All semesters"), ...semesterOptions(taken, termLabel)]);
  const variant = $derived(wide.current ? "compact" : "field");
</script>

{#if personal.mode !== "loading" && taken.length > 0}
  <div class="grid w-full grid-cols-2 gap-3 sm:flex sm:flex-wrap sm:items-center">
    <Select
      label="Department"
      {variant}
      class="col-span-2 sm:w-52"
      options={deptChoices}
      bind:value={takenList.filters.department}
    />
    <Select
      label="Course level"
      {variant}
      class="min-w-0 sm:w-44"
      options={levelChoices}
      bind:value={takenList.filters.level}
    />
    <Select
      label="Semester"
      {variant}
      class="min-w-0 sm:w-44"
      options={semesterChoices}
      bind:value={takenList.filters.semester}
    />
    <div class="col-span-2 flex items-center justify-between gap-3 sm:ml-auto">
      <p class="text-label-md text-muted" aria-live="polite">
        {shown} of {taken.length} {taken.length === 1 ? "course" : "courses"}
      </p>
      <ViewToggle
        pageKey="profile-taken"
        bind:value={
          () => takenList.view ?? storedView("profile-taken") ?? "cards",
          (next: ListView) => (takenList.view = next)
        }
      />
    </div>
  </div>
{/if}
