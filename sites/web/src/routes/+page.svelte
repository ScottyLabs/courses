<script lang="ts">
  import { BookOpen, CalendarDays, Coins, Globe, GraduationCap, Search } from "@lucide/svelte";
  import PageToolbar from "$lib/components/layout/PageToolbar.svelte";
  import RailGroup from "$lib/components/layout/RailGroup.svelte";
  import RailLayout from "$lib/components/layout/RailLayout.svelte";
  import RailSection from "$lib/components/layout/RailSection.svelte";
  import Title from "$lib/components/shell/Title.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import MultiSelect from "$lib/components/ui/MultiSelect.svelte";
  import RangeSlider from "$lib/components/ui/RangeSlider.svelte";
  import SearchInput from "$lib/components/ui/SearchInput.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import ViewToggle, { storedView, type ListView } from "$lib/components/ui/ViewToggle.svelte";

  // Layout only: the explorer feature fills these from the catalog and the URL.
  let query = $state("");
  let view = $state<ListView>(storedView("courses") ?? "cards");
  let departments = $state<string[]>([]);
  let unitsLow = $state(0);
  let unitsHigh = $state(24);
  let offeredIn = $state("");
  let level = $state("");
  let requirements = $state<string[]>([]);

  const unitPresets = [
    { label: "6 Units", low: 6, high: 6 },
    { label: "9 Units", low: 9, high: 9 },
    { label: "12 Units", low: 12, high: 12 },
    { label: "15 Units", low: 15, high: 15 },
  ];
  const levels = [
    { value: "100", label: "100 level" },
    { value: "200", label: "200 level" },
    { value: "300", label: "300 level" },
    { value: "400", label: "400 level" },
    { value: "graduate", label: "Graduate" },
  ];
  const filterCount = $derived(
    departments.length +
      requirements.length +
      (unitsLow > 0 || unitsHigh < 24 ? 1 : 0) +
      (offeredIn === "" ? 0 : 1) +
      (level === "" ? 0 : 1),
  );

  function reset(): void {
    departments = [];
    requirements = [];
    unitsLow = 0;
    unitsHigh = 24;
    offeredIn = "";
    level = "";
  }
</script>

<Title />

<RailLayout storageKey="courses" {filterCount} backLabel="Search">
  {#snippet rail()}
    <RailGroup>
      <RailSection title="Department" icon={GraduationCap} badge={departments.length}>
        <MultiSelect options={[]} bind:value={departments} label="Departments" placeholder="Any department" />
      </RailSection>
      <RailSection title="Units" icon={Coins}>
        <RangeSlider label="Units" bind:low={unitsLow} bind:high={unitsHigh} presets={unitPresets} />
      </RailSection>
      <RailSection title="Offered in" icon={CalendarDays}>
        <Select bind:value={offeredIn} label="Offered in" placeholder="Any semester" options={[]} />
      </RailSection>
      <RailSection title="Course Level" icon={BookOpen}>
        <Select bind:value={level} label="Course level" placeholder="Any level" options={levels} />
      </RailSection>
      <RailSection title="Requirement" icon={Globe} badge={requirements.length}>
        <MultiSelect
          options={[]}
          bind:value={requirements}
          label="Requirements"
          placeholder="Any requirement"
        />
      </RailSection>
      <Button variant="secondary" full onclick={reset}>Reset</Button>
    </RailGroup>
  {/snippet}

  {#snippet header()}
    <PageToolbar title="Search" hideTitle="desktop" onreset={filterCount > 0 ? reset : undefined}>
      {#snippet search()}
        <SearchInput
          bind:value={query}
          label="Search courses"
          placeholder="Search courses by ID, description, name or keyword..."
        />
      {/snippet}
      {#snippet actions()}
        <ViewToggle pageKey="courses" bind:value={view} />
      {/snippet}
    </PageToolbar>
  {/snippet}

  <EmptyState
    icon={Search}
    title="Courses"
    description="Courses for the selected semester are listed here, as cards or as a compact list."
  />
</RailLayout>
