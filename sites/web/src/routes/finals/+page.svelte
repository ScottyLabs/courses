<script lang="ts">
  import { BookOpen, CalendarDays, Clock, Download, GraduationCap, MapPin, Upload } from "@lucide/svelte";
  import PageToolbar from "$lib/components/layout/PageToolbar.svelte";
  import RailGroup from "$lib/components/layout/RailGroup.svelte";
  import RailLayout from "$lib/components/layout/RailLayout.svelte";
  import RailSection from "$lib/components/layout/RailSection.svelte";
  import Title from "$lib/components/shell/Title.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import DataTable from "$lib/components/ui/DataTable.svelte";
  import MultiSelect from "$lib/components/ui/MultiSelect.svelte";
  import SearchInput from "$lib/components/ui/SearchInput.svelte";

  // Layout only: the finals feature fills these from the registrar's schedule.
  let query = $state("");
  let departments = $state<string[]>([]);

  const columns = [
    { key: "course", label: "Course", icon: BookOpen },
    { key: "day", label: "Day", icon: CalendarDays },
    { key: "start", label: "Start Time", icon: Clock },
    { key: "end", label: "End Time", icon: Clock },
    { key: "location", label: "Location", icon: MapPin },
  ];
</script>

<Title value="Finals" />

<RailLayout storageKey="finals" filterCount={departments.length} backLabel="Finals">
  {#snippet rail()}
    <RailGroup>
      <RailSection title="Department" icon={GraduationCap} badge={departments.length}>
        <MultiSelect options={[]} bind:value={departments} label="Departments" placeholder="Any department" />
      </RailSection>
      <Button variant="secondary" full onclick={() => (departments = [])}>Reset</Button>
    </RailGroup>
  {/snippet}

  {#snippet header()}
    <PageToolbar title="Final Exam Schedule">
      {#snippet intro()}
          <ol class="-mt-1 mb-3 list-inside list-decimal text-body-2 text-ink lg:-mt-2">
          <li>Download your calendar export from SIO</li>
          <li>Import the .ics file below</li>
        </ol>
        <div class="mb-4 flex flex-wrap gap-3">
          <Button variant="primary" size="sm" icon={Upload}>Upload Schedule</Button>
          <Button size="sm" icon={Download} href="https://s3.andrew.cmu.edu/sio/index.html">SIO Export</Button>
        </div>
      {/snippet}
      {#snippet search()}
        <SearchInput
          bind:value={query}
          label="Search final exams"
          placeholder="Search courses by ID, description, name or keyword..."
        />
      {/snippet}
    </PageToolbar>
  {/snippet}

  <DataTable
    {columns}
    rows={[]}
    variant="outline"
    caption="Final exam schedule"
    empty="Final exams for the selected semester are listed here."
  />
</RailLayout>
