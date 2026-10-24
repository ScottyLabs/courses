<script lang="ts">
  // CMU courses on the record: add one (or paste a list), then see them as cards or a compact list, grouped by
  // semester, with grades editable. The department, level and semester filters and the cards/list switch are in the
  // profile's title band (TakenFilters). Courseweave's TakenSection (features/record/TakenSection.tsx), shown the way
  // the Figma's Courses Taken page shows courses.
  import { Plus } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import TextInput from "$lib/components/ui/TextInput.svelte";
  import { storedView } from "$lib/components/ui/ViewToggle.svelte";
  import type { Option } from "$lib/components/ui/types";
  import { currentTerm } from "$lib/core/degree/hooks";
  import { parsePastedCourses, takenEntry, unitsError } from "$lib/core/record/forms";
  import { groupByTerm, matchesFilters, takenEntries } from "$lib/core/record/taken-list";
  import { codeOf } from "$lib/core/requisite-format";
  import { termLabel } from "$lib/core/requisites";
  import { entryUnits } from "$lib/core/state/record";
  import { record } from "$lib/state";
  import CoursePicker from "./CoursePicker.svelte";
  import { GRADE_OPTIONS } from "$lib/core/record/grades";
  import type { RecordData } from "./record-data.svelte";
  import RecordSection from "./RecordSection.svelte";
  import TakenCard from "./TakenCard.svelte";
  import TakenRow from "./TakenRow.svelte";
  import { clearTakenFilters, takenFiltered, takenList } from "./taken-list.svelte";

  let { data }: { data: RecordData } = $props();

  // ---- Adding ----
  let courseId = $state<string | null>(null);
  let termCode = $state(currentTerm());
  let grade = $state("");
  let units = $state("");
  let paste = $state("");
  let pasteOpen = $state(false);
  let message = $state<string | null>(null);

  const taken = $derived(takenEntries(record.entries));
  const termChoices = $derived.by(() => {
    const options = data.termChoices(record.profile.entryTerm);
    return termCode !== "" && !options.includes(termCode) ? [termCode, ...options] : options;
  });
  const termSelectOptions = $derived<Option[]>([
    ...termChoices.map((t) => ({ value: t, label: termLabel(t) })),
    { value: "", label: "Not sure" },
  ]);
  const picked = $derived(courseId === null ? null : data.lookup(courseId));
  const duplicate = $derived(
    courseId !== null && taken.some((e) => e.courseId === courseId && (e.term ?? "") === termCode),
  );
  const draft = $derived(
    takenEntry({ courseId, term: termCode, grade, units, catalogUnits: picked?.units }),
  );
  const unitsMessage = $derived(unitsError(units));
  const pasted = $derived(parsePastedCourses(paste));
  const forTerm = $derived(termCode === "" ? "" : ` for ${termLabel(termCode)}`);

  function add(): void {
    if (draft?.courseId === null || draft?.courseId === undefined) return;
    record.addEntry(draft);
    message = `Added ${codeOf(draft.courseId)}${forTerm}.`;
    courseId = null;
    grade = "";
    units = "";
  }

  function addPasted(): void {
    const fresh = pasted.filter(
      (id) => !taken.some((e) => e.courseId === id && (e.term ?? "") === termCode),
    );
    record.addEntries(
      fresh.map((id) => ({ courseId: id, source: "taken" as const, ...(termCode === "" ? {} : { term: termCode }) })),
    );
    message =
      fresh.length > 0
        ? `Added ${fresh.length} ${fresh.length === 1 ? "course" : "courses"}${forTerm}.`
        : "Those courses are already on your record for that semester.";
    paste = "";
    pasteOpen = false;
  }

  // ---- Showing ----
  const facts = (id: string): ReturnType<RecordData["facts"]> => data.facts(id);
  const shown = $derived(taken.filter((e) => matchesFilters(e, takenList.filters, facts)));
  const groups = $derived(groupByTerm(shown));
  const view = $derived(takenList.view ?? storedView("profile-taken") ?? "cards");
  const fmt = (n: number): string => (Number.isInteger(n) ? String(n) : n.toFixed(1));
</script>

<RecordSection
  id="record-taken"
  title="Courses taken"
  description="CMU courses you've finished or are taking now. Grades are optional; without one, a course counts as passed."
>
  <div class="flex flex-col gap-4 rounded-card border border-pressed bg-surface p-5">
    <div class="grid grid-cols-1 gap-4 md:grid-cols-[minmax(0,1fr)_160px_170px_96px]">
      <CoursePicker
        label="Course"
        bind:value={courseId}
        {data}
        onpick={(id) => {
          message = null;
          units = id === null ? "" : String(data.lookup(id).units ?? "");
        }}
      />
      <Select label="Semester" hideLabel={false} options={termSelectOptions} bind:value={termCode} />
      <Select label="Grade" hideLabel={false} options={GRADE_OPTIONS} bind:value={grade} />
      <TextInput
        label="Units"
        inputmode="decimal"
        bind:value={units}
        placeholder={picked !== null && picked.units === null ? "VAR" : ""}
        error={unitsMessage ?? undefined}
      />
    </div>
    <div class="flex flex-wrap items-center gap-2">
      <Button variant="primary" icon={Plus} disabled={draft === null} onclick={add}>Add course</Button>
      <Button variant="ghost" aria-expanded={pasteOpen} onclick={() => (pasteOpen = !pasteOpen)}>
        Paste a list instead
      </Button>
      <p class="text-body-2" aria-live="polite">
        {#if duplicate}
          <span class="text-warn">Already on your record{forTerm}.</span>
        {:else if message !== null}
          <span class="text-ok">{message}</span>
        {/if}
      </p>
    </div>
    {#if pasteOpen}
      <div class="flex flex-col gap-2 border-t border-line pt-4">
        <label for="record-paste" class="text-label-lg text-ink">
          Course numbers for {termCode === "" ? "an unknown semester" : termLabel(termCode)}
        </label>
        <textarea
          id="record-paste"
          bind:value={paste}
          rows="3"
          aria-describedby="record-paste-hint"
          placeholder="15-122, 21-127, 76-101"
          class="w-full rounded-field border border-line-strong bg-surface px-4 py-2 text-body-2 text-ink outline-none placeholder:text-muted hover:border-muted focus:border-accent"
        ></textarea>
        <p id="record-paste-hint" class="text-label-md font-normal text-muted">
          Paste from SIO or type them: 15-122, 21-127, 76101. Grades can be added after.
        </p>
        <div class="flex flex-wrap items-center gap-2">
          <Button variant="secondary" disabled={pasted.length === 0} onclick={addPasted}>
            Add {pasted.length > 0 ? pasted.length : ""}
            {pasted.length === 1 ? "course" : "courses"}
          </Button>
          {#if pasted.length > 0}
            <span class="tabular min-w-0 text-body-2 text-muted">{pasted.map((id) => codeOf(id)).join(", ")}</span>
          {/if}
        </div>
      </div>
    {/if}
  </div>

  {#if taken.length === 0}
    <p class="rounded-card border border-dashed border-line-strong px-4 py-5 text-body-2 text-muted">
      Nothing here yet. Add the courses you've taken so prerequisites can be checked in your schedules and degree plan.
    </p>
  {:else if shown.length === 0}
    <div class="flex flex-wrap items-center gap-3 rounded-card border border-dashed border-line-strong px-4 py-5">
      <p class="text-body-2 text-muted">No courses on your record match the filters at the top of the page.</p>
      {#if takenFiltered()}
        <Button size="sm" variant="secondary" onclick={clearTakenFilters}>Clear filters</Button>
      {/if}
    </div>
  {:else}
    <div class="flex flex-col gap-6">
      {#each groups as group (group.term)}
        {@const total = group.entries.reduce(
          (n, e) => n + entryUnits(e, e.courseId === null ? null : data.lookup(e.courseId).units),
          0,
        )}
        <section aria-labelledby="taken-{group.term || 'none'}">
          <div class="mb-2 flex items-baseline justify-between gap-3">
            <h3 id="taken-{group.term || 'none'}" class="text-heading-6 text-strong">
              {group.term === "" ? "Semester not set" : termLabel(group.term)}
            </h3>
            <span class="tabular text-body-2 text-muted">{fmt(total)} units</span>
          </div>
          {#if view === "cards"}
            <ul class="grid grid-cols-1 gap-4 lg:grid-cols-2">
              {#each group.entries as entry (entry.id)}
                <li class="flex *:flex-1"><TakenCard {entry} {data} /></li>
              {/each}
            </ul>
          {:else}
            <ul class="divide-y divide-line overflow-hidden rounded-card border border-pressed bg-surface">
              {#each group.entries as entry (entry.id)}
                <TakenRow {entry} {data} />
              {/each}
            </ul>
          {/if}
        </section>
      {/each}
    </div>
  {/if}
</RecordSection>
