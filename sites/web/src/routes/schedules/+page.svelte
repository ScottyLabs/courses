<script lang="ts">
  import { CalendarDays, PencilLine, Plus } from "@lucide/svelte";
  import PageToolbar from "$lib/components/layout/PageToolbar.svelte";
  import RailGroup from "$lib/components/layout/RailGroup.svelte";
  import RailLayout from "$lib/components/layout/RailLayout.svelte";
  import Title from "$lib/components/shell/Title.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import SearchInput from "$lib/components/ui/SearchInput.svelte";
  import Select from "$lib/components/ui/Select.svelte";

  // Layout only: the schedules feature fills these from the student's schedules.
  let query = $state("");
  let schedule = $state("");
  let semester = $state("");
</script>

<Title value="Schedules" />

<RailLayout title="Schedule" name="schedule panel" sheetLabel="Schedule" backLabel="Back" storageKey="schedules">
  {#snippet titleAction()}
    <IconButton label="New schedule" icon={Plus} size="sm" />
  {/snippet}
  {#snippet rail()}
    <RailGroup>
      <Select bind:value={schedule} label="Schedule" hideLabel={false} placeholder="Select schedule" options={[]} />
    </RailGroup>
    <RailGroup title="Schedule Calendar">
      <Select bind:value={semester} label="Semester" hideLabel={false} placeholder="Select semester" options={[]} />
    </RailGroup>
  {/snippet}

  {#snippet header()}
    <PageToolbar title="Untitled">
      {#snippet titleAction()}
        <IconButton label="Rename schedule" icon={PencilLine} size="sm" />
      {/snippet}
      {#snippet search()}
        <SearchInput bind:value={query} label="Add a course" placeholder="Search courses to add by ID or name..." />
      {/snippet}
    </PageToolbar>
  {/snippet}

  <EmptyState
    icon={CalendarDays}
    title="Week calendar"
    description="The schedule's courses and sections appear on a week calendar here, with conflicts, unit and workload totals."
  />
</RailLayout>
