<script lang="ts">
  // A course on the record as a small course card (Figma "Profile Page (Courses Taken)"): code and title, department,
  // units and FCE hours, the semester, and the grade, which can be changed here.
  import { Clock, IterationCcw, SquareArrowOutUpRight, Trash2 } from "@lucide/svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import { formatHours } from "$lib/core/fce";
  import { departmentLabel, departmentKey } from "$lib/core/record/taken-list";
  import { termLabel } from "$lib/core/requisites";
  import { coursePath } from "$lib/core/routes";
  import { entryUnits, type RecordEntry } from "$lib/core/state/record";
  import { record } from "$lib/state";
  import { gradeNote, gradeOptionsWith } from "./grades";
  import type { RecordData } from "./record-data.svelte";

  let { entry, data }: { entry: RecordEntry; data: RecordData } = $props();

  const id = $derived(entry.courseId ?? "");
  const info = $derived(data.lookup(id));
  const facts = $derived(data.facts(id));
  const units = $derived(entryUnits(entry, info.units));
  const hours = $derived(data.hours(id));
  const note = $derived(gradeNote(entry.grade, info));
  const href = $derived(coursePath(id, entry.term));
  const fmt = (n: number): string => (Number.isInteger(n) ? String(n) : n.toFixed(1));
</script>

<article class="flex min-w-0 flex-col gap-2 rounded-card border border-pressed bg-surface p-4 sm:px-5">
  <div class="flex items-start gap-2">
    <h3 class="min-w-0 flex-1 text-heading-6 text-strong">
      <a {href} class="hover:underline">
        <span class="tabular font-semibold">{info.code}</span>
        <span class="ml-1.5 font-normal text-ink">{entry.label ?? info.title ?? "Not in the catalog"}</span>
      </a>
    </h3>
    <IconButton {href} label="Open {info.code}" icon={SquareArrowOutUpRight} class="-mt-2 -mr-2" />
  </div>
  {#if facts.dept}
    <p class="-mt-1 text-body-2 text-muted">{departmentLabel(departmentKey(id, facts))}</p>
  {/if}
  <p class="flex flex-wrap items-center gap-x-3 gap-y-1 text-body-2 text-ink">
    <span class="inline-flex items-center gap-1.5">
      <IterationCcw aria-hidden="true" class="size-4" />
      <span class="tabular">{fmt(units)} units</span>
    </span>
    {#if hours !== null}
      <span aria-hidden="true" class="h-4 w-px bg-line-strong"></span>
      <span class="inline-flex items-center gap-1.5">
        <Clock aria-hidden="true" class="size-4" />
        <span class="tabular">{formatHours(hours)} hrs/week</span>
      </span>
    {/if}
  </p>
  <div class="mt-auto flex items-end gap-2 border-t border-line pt-3">
    <Select
      label="Grade for {info.code}"
      class="min-w-0 flex-1 sm:max-w-[200px]"
      options={gradeOptionsWith(entry.grade)}
      value={entry.grade ?? ""}
      onchange={(event) =>
        record.updateEntry(entry.id, { grade: event.currentTarget.value === "" ? undefined : event.currentTarget.value })}
    />
    <span class="ml-auto self-center text-label-md text-muted">
      {entry.term ? termLabel(entry.term) : "Semester not set"}
    </span>
    <IconButton label="Remove {info.code} from your record" icon={Trash2} onclick={() => record.removeEntry(entry.id)} />
  </div>
  {#if note}<p class="text-label-md font-normal text-muted">{note}</p>{/if}
</article>
