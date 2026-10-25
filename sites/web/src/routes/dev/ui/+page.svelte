<script lang="ts">
  import {
    BookmarkPlus,
    Calendar1,
    Clock,
    Coins,
    Download,
    ExternalLink,
    Link,
    LogOut,
    MoreHorizontal,
    Plus,
    Settings,
    Share2,
    SlidersHorizontal,
    Star,
    Trash2,
    Upload,
    Users,
  } from "@lucide/svelte";
  import PageContainer from "$lib/components/layout/PageContainer.svelte";
  import Title from "$lib/components/shell/Title.svelte";
  import {
    Avatar,
    AvatarGroup,
    Button,
    Card,
    Checkbox,
    Chip,
    Combobox,
    CourseChip,
    DataTable,
    Dialog,
    EmptyState,
    IconButton,
    Menu,
    MultiSelect,
    Popover,
    RangeSlider,
    SearchInput,
    SegmentedControl,
    Select,
    Sheet,
    Skeleton,
    Spinner,
    StarRating,
    StatBox,
    StatusChip,
    Switch,
    Tabs,
    TextInput,
    ToggleChips,
    Tooltip,
    ViewToggle,
    toast,
    type ListView,
  } from "$lib/components/ui";

  let layout = $state<"side" | "light" | "dark">("side");

  function panelState() {
    return {
      text: "",
      short: "",
      search: "",
      select: "",
      compact: "",
      department: null as string | null,
      chosen: [] as string[],
      checked: true,
      unchecked: false,
      mixed: true,
      on: true,
      off: false,
      low: 0,
      high: 12,
      days: ["M", "W"],
      tab: "overview",
      segment: "week",
      view: "cards" as ListView,
      dialog: false,
      sheet: false,
      bottom: false,
      popover: false,
    };
  }

  // Each theme panel has its own state so the two copies can be compared independently.
  const panels = $state({ light: panelState(), dark: panelState() });

  const tokens = [
    "paper",
    "surface",
    "surface-2",
    "ink",
    "strong",
    "muted",
    "faint",
    "line",
    "line-strong",
    "pressed",
    "accent",
    "accent-ink",
    "ok-bg",
    "ok",
    "warn-bg",
    "warn",
    "danger-bg",
    "danger",
  ];
  const typeScale = [
    ["text-heading-3", "Heading 3, 24/32 medium"],
    ["text-heading-4", "Heading 4, 20/28 medium"],
    ["text-heading-5", "Heading 5, 18/26 medium"],
    ["text-heading-6", "Heading 6, 16/24 medium"],
    ["text-body-1", "Body 1, 16/24 regular"],
    ["text-body-2", "Body 2, 14/20 regular"],
    ["text-label-lg", "Label large, 14/20 semibold"],
    ["text-label-md", "Label medium, 12/16 medium"],
    ["text-label-sm", "Label small, 10/14 regular"],
  ];
  const departments = [
    { value: "15", label: "Computer Science", keywords: "SCS CS" },
    { value: "21", label: "Mathematical Sciences", keywords: "MCS math" },
    { value: "18", label: "Electrical and Computer Engineering", keywords: "ECE CIT" },
    { value: "36", label: "Statistics and Data Science", keywords: "Dietrich" },
    { value: "76", label: "English", keywords: "Dietrich" },
    { value: "05", label: "Human-Computer Interaction", keywords: "HCII SCS" },
    { value: "10", label: "Machine Learning", keywords: "MLD SCS" },
    { value: "33", label: "Physics", keywords: "MCS" },
  ];
  const requirements = [
    { value: "cs-core", label: "CS Core", description: "Computer Science, BS" },
    { value: "math", label: "Mathematics", description: "Computer Science, BS" },
    { value: "humanities", label: "Humanities and Arts", description: "SCS general education" },
    { value: "science", label: "Science and Engineering", description: "SCS general education" },
  ];
  const levels = [
    { value: "100", label: "100 level" },
    { value: "200", label: "200 level" },
    { value: "300", label: "300 level" },
    { value: "graduate", label: "Graduate" },
  ];
  const fceColumns = [
    { key: "semester", label: "Semester" },
    { key: "lecture", label: "Lecture" },
    { key: "instructor", label: "Instructor" },
    { key: "workload", label: "Workload" },
    { key: "teaching", label: "Teaching" },
    { key: "course", label: "Course Rate" },
    { key: "respondents", label: "Respondents" },
    { key: "rate", label: "Response Rate" },
  ];
  const fceRows = [
    { semester: "Fall 2024", lecture: 2, instructor: "CERVESATO, ILIANO", workload: 17.1, teaching: 3.48, course: 3.49, respondents: 96, rate: 49.5 },
    { semester: "Fall 2024", lecture: 1, instructor: "CERVESATO, ILIANO", workload: 13.58, teaching: 3.44, course: 3.69, respondents: 145, rate: 61.4 },
    { semester: "Fall 2024", lecture: 2, instructor: "KOHLBRENNER, ANNE", workload: 17.13, teaching: 4.14, course: 3.82, respondents: 96, rate: 49.5 },
  ];
  const finalColumns = [
    { key: "course", label: "Course" },
    { key: "day", label: "Day" },
    { key: "start", label: "Start Time" },
    { key: "end", label: "End Time" },
  ];
  const finalRows = [
    { course: "48321A", day: "Monday, December 8", start: "8:30 AM", end: "11:30 AM" },
    { course: "48353A", day: "Monday, December 8", start: "1:00 PM", end: "4:00 PM" },
  ];
  const friends = [{ name: "Chris Tan" }, { name: "Sam Lee" }, { name: "Ana Diaz" }, { name: "Raj Patel" }];
  const tabs = [
    { id: "overview", label: "Overview" },
    { id: "sections", label: "Sections", count: 4 },
    { id: "fces", label: "FCEs" },
    { id: "syllabi", label: "Syllabi", disabled: true },
  ];
  const menuItems = [
    { label: "Rename", icon: Settings, onselect: () => toast("Rename chosen") },
    { label: "Share link", icon: Share2, onselect: () => toast("Link copied", { kind: "success" }) },
    "separator" as const,
    { label: "Delete schedule", icon: Trash2, danger: true, onselect: () => toast("Deleted", { kind: "error" }) },
  ];
</script>

<Title value="Component gallery" />

{#snippet gallery(mode: string)}
  {@const state = panels[mode === "dark" ? "dark" : "light"]}
  <div class="{mode} flex flex-col gap-10 rounded-card border border-line bg-paper p-5 text-ink sm:p-8">
    <h2 class="text-heading-4 text-strong">{mode === "dark" ? "Dark" : "Light"}</h2>

    <section class="flex flex-col gap-4">
      <h3 class="text-heading-5 text-strong">Color tokens</h3>
      <div class="grid grid-cols-2 gap-3 sm:grid-cols-3">
        {#each tokens as token (token)}
          <div class="flex items-center gap-3">
            <span class="size-8 shrink-0 rounded-[8px] border border-line" style="background: var(--color-{token})"></span>
            <code class="text-label-md">{token}</code>
          </div>
        {/each}
      </div>
    </section>

    <section class="flex flex-col gap-3">
      <h3 class="text-heading-5 text-strong">Type</h3>
      {#each typeScale as [cls, label] (cls)}
        <p class={cls}>{label}</p>
      {/each}
    </section>

    <section class="flex flex-col gap-4">
      <h3 class="text-heading-5 text-strong">Buttons</h3>
      <div class="flex flex-wrap items-center gap-3">
        <Button variant="primary" icon={Upload}>Upload SIO calendar</Button>
        <Button variant="secondary">Reset</Button>
        <Button icon={LogOut}>Sign out</Button>
        <Button variant="ghost" icon={Plus}>Add</Button>
        <Button variant="danger" icon={Trash2}>Delete</Button>
      </div>
      <div class="flex flex-wrap items-center gap-3">
        <Button size="sm" icon={Link}>Share link</Button>
        <Button size="sm" icon={Download}>Export from SIO</Button>
        <Button size="lg" variant="primary">Large</Button>
        <Button loading>Saving</Button>
        <Button disabled>Disabled</Button>
        <Button href="https://git.cmu.dev/ScottyLabs/courses" trailingIcon={ExternalLink} variant="ghost">Link</Button>
      </div>
      <div class="flex flex-wrap items-center gap-2">
        <IconButton label="Save course" icon={Star} />
        <IconButton label="Save course" icon={Star} pressed />
        <IconButton label="Filters" icon={SlidersHorizontal} variant="outline" />
        <IconButton label="Add" icon={Plus} variant="solid" />
        <IconButton label="More" icon={MoreHorizontal} size="sm" />
        <IconButton label="Settings" icon={Settings} size="lg" disabled />
      </div>
    </section>

    <section class="flex flex-col gap-4">
      <h3 class="text-heading-5 text-strong">Fields</h3>
      <SearchInput bind:value={state.search} label="Search courses" placeholder="Search courses by ID, description, name or keyword..." />
      <SearchInput bind:value={state.search} variant="field" label="Search friends" placeholder="Search friends" />
      <div class="grid gap-4 sm:grid-cols-2">
        <TextInput bind:value={state.text} label="Schedule name" placeholder="Untitled" hint="Shown on shared links." />
        <TextInput bind:value={state.text} label="Andrew ID" placeholder="acarnegie" error="No student has this Andrew ID." />
        <TextInput bind:value={state.short} label="Semesters to show" size="sm" placeholder="None" />
        <TextInput value="Locked" label="Disabled" disabled />
        <Select bind:value={state.select} label="Course level" hideLabel={false} placeholder="None" options={levels} />
        <Select value="" label="GenEd category" hideLabel={false} placeholder="Select GenEd Category" options={levels} disabled />
        <Select bind:value={state.compact} label="Show" variant="compact" placeholder="Show" options={levels} />
        <Combobox options={departments} bind:value={state.department} label="Department" hideLabel={false} placeholder="Any department" />
        <MultiSelect options={requirements} bind:value={state.chosen} label="Requirements" hideLabel={false} placeholder="Any requirement" />
      </div>
      <div class="flex flex-wrap items-center gap-x-6">
        <Checkbox bind:checked={state.checked} label="Spring" />
        <Checkbox bind:checked={state.unchecked} label="Summer" />
        <Checkbox bind:checked={state.mixed} indeterminate label="All minis" />
        <Checkbox checked disabled label="Disabled" />
        <Checkbox bind:checked={state.unchecked} label="Pass/fail" description="Counts toward units, not QPA" />
      </div>
      <div class="flex flex-wrap items-center gap-6">
        <Switch bind:checked={state.on} label="Show weekends" />
        <Switch bind:checked={state.off} label="Hide conflicts" labelPosition="end" />
        <Switch checked label="Disabled" disabled />
      </div>
      <div class="max-w-[356px]">
        <RangeSlider
          label="Units"
          bind:low={state.low}
          bind:high={state.high}
          presets={[
            { label: "6 Units", low: 6, high: 6 },
            { label: "9 Units", low: 9, high: 9 },
            { label: "12 Units", low: 12, high: 12 },
            { label: "15 Units", low: 15, high: 15 },
          ]}
        />
      </div>
      <ToggleChips
        label="Days"
        bind:value={state.days}
        options={[
          { value: "M", label: "Mon" },
          { value: "T", label: "Tue" },
          { value: "W", label: "Wed" },
          { value: "R", label: "Thu" },
          { value: "F", label: "Fri" },
        ]}
      />
    </section>

    <section class="flex flex-col gap-4">
      <h3 class="text-heading-5 text-strong">Chips</h3>
      <div class="flex flex-wrap items-center gap-2">
        <Chip label="Dietrich" onremove={() => toast("Removed Dietrich")} />
        <Chip label="12 Units" onremove={() => toast("Removed 12 Units")} />
        <Chip tone="soft" icon={ExternalLink} href="https://csd.cmu.edu" label="SCS Dept" />
        <Chip tone="soft" size="xs" icon={ExternalLink} href="https://csd.cmu.edu" label="Course Website" />
        <Chip tone="neutral" size="sm" icon={Users} label="SCS First Year" />
        <Chip tone="accent" shape="pill" class="px-4" onclick={() => toast("Reset")}>Reset</Chip>
        <Chip tone="strong" shape="pill" label="Filter 3" />
      </div>
      <div class="flex flex-wrap items-center gap-2">
        <StatusChip status="satisfied" label="15-122" />
        <StatusChip status="planned" label="15-150" />
        <StatusChip status="in-progress" label="21-127" />
        <StatusChip status="unsatisfied" label="21-128" />
        <CourseChip code="05-360" />
        <CourseChip code="15-122" tone="soft" />
        <CourseChip code="15-122" title="Principles of Imperative Computation" tone="soft" />
        <CourseChip code="15-251" status="satisfied" />
      </div>
    </section>

    <section class="flex flex-col gap-4">
      <h3 class="text-heading-5 text-strong">Cards and data</h3>
      <Card padding="md" interactive>
        <div class="flex items-start justify-between gap-4">
          <div class="flex flex-col gap-2">
            <p class="text-body-1"><span class="font-bold">15-122</span> <span class="font-medium">Principles of Imperative Computation</span></p>
            <p class="-mt-1 text-label-lg text-muted">Computer Science</p>
            <p class="flex items-center gap-3 text-label-lg">
              <span class="inline-flex items-center gap-1"><Coins aria-hidden="true" class="size-[18px]" />12 units</span>
              <span class="h-5 w-0.5 bg-line"></span>
              <span class="inline-flex items-center gap-2"><Clock aria-hidden="true" class="size-[18px]" />15.5 hrs/week</span>
            </p>
          </div>
          <div class="flex flex-col items-end gap-2">
            <IconButton label="Open course" icon={ExternalLink} size="sm" href="/course/15-122" />
            <AvatarGroup people={friends} size="xs" label="Friends taking this course" />
          </div>
        </div>
      </Card>
      <div class="flex flex-wrap items-center gap-4">
        <Avatar name="Andrew Carnegie" size="lg" />
        <Avatar name="Kosbie, David" size="md" />
        <Avatar size="md" />
        <Avatar name="Chris Tan" size="sm" tone="soft" letters={1} />
        <Avatar name="Chris Tan" size="xs" />
        <AvatarGroup people={friends} label="Friends taking this course" />
      </div>
      <div class="flex flex-wrap items-center gap-6">
        <StarRating value={3.91} label="Teaching rate" />
        <StarRating value={4.5} size="sm" label="Course rate" />
      </div>
      <div class="grid gap-3 sm:grid-cols-3">
        <StatBox value="15.19" unit="hrs/wk" label="Workload" />
        <StatBox label="Teaching Rate"><StarRating value={3.91} label="Teaching rate" /></StatBox>
        <StatBox label="Course Rate"><StarRating value={3.81} label="Course rate" /></StatBox>
      </div>
      <DataTable columns={fceColumns} rows={fceRows} caption="FCE results by semester">
        {#snippet title()}
          <p class="text-heading-6 text-strong">Aggregate Data <span class="text-body-2 text-muted">(data from 1 semesters)</span></p>
          <div class="mt-3 grid gap-2 sm:grid-cols-3">
            <StatBox variant="plain" value="15.19" unit="hrs/wk" label="Workload" />
            <StatBox variant="plain" label="Teaching Rate"><StarRating value={3.91} label="Teaching rate" /></StatBox>
            <StatBox variant="plain" label="Course Rate"><StarRating value={3.81} label="Course rate" /></StatBox>
          </div>
        {/snippet}
        {#snippet cell(row, column)}
          {#if column.key === "instructor"}
            <a href="/instructors">{row.instructor}</a>
          {:else}
            {row[column.key as keyof typeof row]}
          {/if}
        {/snippet}
      </DataTable>
      <DataTable columns={finalColumns} rows={finalRows} variant="outline" caption="Final exams">
        {#snippet cell(row, column)}
          {#if column.key === "course"}
            <a href="/course/48-321">{row.course}</a>
          {:else}
            {row[column.key as keyof typeof row]}
          {/if}
        {/snippet}
      </DataTable>
    </section>

    <section class="flex flex-col gap-4">
      <h3 class="text-heading-5 text-strong">Overlays</h3>
      <div class="flex flex-wrap items-center gap-3">
        <Button onclick={() => (state.dialog = true)}>Dialog</Button>
        <Button onclick={() => (state.sheet = true)}>Sheet</Button>
        <Button onclick={() => (state.bottom = true)}>Bottom sheet</Button>
        <Popover bind:open={state.popover} label="Share schedule" class="w-72 p-4">
          {#snippet trigger(props)}
            <Button icon={Share2} {...props}>Popover</Button>
          {/snippet}
          {#snippet children({ close })}
            <p class="text-heading-6 text-strong">Share schedule</p>
            <p class="mt-1 mb-3 text-body-2 text-muted">Anyone with the link can view it.</p>
            <div class="flex justify-end gap-2">
              <Button size="sm" variant="ghost" onclick={() => close()}>Cancel</Button>
              <Button size="sm" variant="primary" onclick={() => close()}>Copy link</Button>
            </div>
          {/snippet}
        </Popover>
        <Menu items={menuItems} label="Schedule actions">
          {#snippet trigger(props)}
            <IconButton label="Schedule actions" icon={MoreHorizontal} variant="outline" {...props} />
          {/snippet}
        </Menu>
        <Tooltip text="Saved courses are kept in your account">
          {#snippet trigger(props)}
            <IconButton label="Save course" icon={BookmarkPlus} {...props} tooltip={false} />
          {/snippet}
        </Tooltip>
        <Button size="sm" onclick={() => toast("Schedule saved", { kind: "success" })}>Toast</Button>
        <Button size="sm" onclick={() => toast("Couldn't save. Check your connection.", { kind: "error", action: { label: "Retry", run: () => toast("Retrying") } })}>Error toast</Button>
      </div>
      <Dialog bind:open={state.dialog} title="Delete schedule?" description="This removes the schedule from every device." size="sm">
        <p class="text-body-2">"Fall plan B" and its 6 courses will be deleted.</p>
        {#snippet footer()}
          <Button variant="ghost" onclick={() => (state.dialog = false)}>Cancel</Button>
          <Button variant="danger" icon={Trash2} onclick={() => (state.dialog = false)}>Delete</Button>
        {/snippet}
      </Dialog>
      <Sheet bind:open={state.sheet} title="Filter" backLabel="Search">
        <div class="flex flex-col gap-4 px-5 py-4">
          <Combobox options={departments} bind:value={state.department} label="Department" hideLabel={false} />
          <MultiSelect options={requirements} bind:value={state.chosen} label="Requirements" hideLabel={false} />
        </div>
      </Sheet>
      <Sheet bind:open={state.bottom} title="Section" side="bottom">
        <div class="px-5 py-4 text-body-2">A bottom sheet for short choices on phones.</div>
      </Sheet>
    </section>

    <section class="flex flex-col gap-4">
      <h3 class="text-heading-5 text-strong">Navigation</h3>
      <Tabs {tabs} bind:value={state.tab} label="Course sections">
        {#snippet panel(id)}
          <p class="text-body-2 text-muted">Panel: {id}</p>
        {/snippet}
      </Tabs>
      <div class="flex flex-wrap items-center gap-4">
        <SegmentedControl
          label="Calendar range"
          bind:value={state.segment}
          options={[
            { value: "week", label: "Week" },
            { value: "day", label: "Day" },
            { value: "list", label: "List" },
          ]}
        />
        <ViewToggle pageKey="gallery-{mode}" bind:value={state.view} />
        <span class="text-label-md text-muted">View: {state.view}</span>
      </div>
    </section>

    <section class="flex flex-col gap-4">
      <h3 class="text-heading-5 text-strong">Feedback</h3>
      <div class="flex items-center gap-4">
        <Spinner size="sm" />
        <Spinner />
        <Spinner size="lg" />
      </div>
      <Card padding="md" class="flex flex-col gap-3">
        <Skeleton class="h-6 w-2/3" />
        <Skeleton variant="text" lines={3} />
        <div class="flex gap-2"><Skeleton variant="circle" class="size-6" /><Skeleton variant="circle" class="size-6" /></div>
      </Card>
      <Card padding="none">
        <EmptyState size="sm" icon={Calendar1} title="No courses this semester" description="Courses you add to a schedule appear here.">
          <Button size="sm" variant="primary" icon={Plus}>Add a course</Button>
        </EmptyState>
      </Card>
    </section>
  </div>
{/snippet}

<PageContainer width="wide">
  <div class="mb-6 flex flex-wrap items-center justify-between gap-4">
    <div>
      <h1 class="text-heading-3 text-strong">Component gallery</h1>
      <p class="text-body-2 text-muted">Every primitive in src/lib/components/ui, in both themes. Development builds only.</p>
    </div>
    <SegmentedControl
      label="Themes shown"
      bind:value={layout}
      options={[
        { value: "side", label: "Side by side" },
        { value: "light", label: "Light" },
        { value: "dark", label: "Dark" },
      ]}
    />
  </div>
  <div class="grid gap-6 {layout === 'side' ? 'xl:grid-cols-2' : ''}">
    {#if layout !== "dark"}{@render gallery("light")}{/if}
    {#if layout !== "light"}{@render gallery("dark")}{/if}
  </div>
</PageContainer>
