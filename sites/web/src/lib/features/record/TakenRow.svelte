<script lang="ts">
  // A course on the record as a compact row, for the list view (Courseweave's TakenRow in
  // features/record/TakenSection.tsx): code and title, grade, units and remove.
  import { Trash2 } from "@lucide/svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import { coursePath } from "$lib/core/routes";
  import { entryUnits, type RecordEntry } from "$lib/core/state/record";
  import { record } from "$lib/state";
  import { gradeNote, gradeOptionsWith } from "$lib/core/record/grades";
  import type { RecordData } from "./record-data.svelte";

  let { entry, data }: { entry: RecordEntry; data: RecordData } = $props();

  const id = $derived(entry.courseId ?? "");
  const info = $derived(data.lookup(id));
  const units = $derived(entryUnits(entry, info.units));
  const note = $derived(gradeNote(entry.grade, info));
  const fmt = (n: number): string => (Number.isInteger(n) ? String(n) : n.toFixed(1));
</script>

<li
  class="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-x-3 gap-y-2 px-4 py-2.5 md:grid-cols-[minmax(0,1fr)_170px_80px_auto]"
>
  <div class="min-w-0">
    <p class="flex min-w-0 items-baseline gap-2">
      <a href={coursePath(id, entry.term)} class="tabular shrink-0 text-label-lg text-strong hover:underline">
        {info.code}
      </a>
      <span class="min-w-0 truncate text-body-2 text-ink">{entry.label ?? info.title ?? "Not in the catalog"}</span>
    </p>
    {#if note}<p class="text-label-md font-normal text-muted">{note}</p>{/if}
  </div>
  <IconButton
    label="Remove {info.code} from your record"
    icon={Trash2}
    class="md:order-last"
    onclick={() => record.removeEntry(entry.id)}
  />
  <div class="col-span-2 flex items-center gap-3 md:col-span-1">
    <Select
      label="Grade for {info.code}"
      class="max-w-[200px] flex-1 md:max-w-none"
      options={gradeOptionsWith(entry.grade)}
      value={entry.grade ?? ""}
      onchange={(event) =>
        record.updateEntry(entry.id, {
          grade: event.currentTarget.value === "" ? undefined : event.currentTarget.value,
        })}
    />
    <span class="tabular ml-auto text-body-2 text-ink md:hidden">{fmt(units)} units</span>
  </div>
  <span class="tabular hidden text-right text-body-2 text-ink md:block">{fmt(units)} units</span>
</li>
