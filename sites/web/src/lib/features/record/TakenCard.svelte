<script lang="ts">
  // A course on the record as the shared small course card (Figma "Profile Page (Courses Taken)"): code and title,
  // department, units and FCE hours, then the grade, which can be changed here, the semester and remove.
  import { Trash2 } from "@lucide/svelte";
  import CourseCardSmall from "$lib/components/course/CourseCardSmall.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import { gradeNote, gradeOptionsWith } from "$lib/core/record/grades";
  import { departmentKey, departmentLabel } from "$lib/core/record/taken-list";
  import { termLabel } from "$lib/core/requisites";
  import { entryUnits, type RecordEntry } from "$lib/core/state/record";
  import { record } from "$lib/state";
  import type { RecordData } from "./record-data.svelte";

  let { entry, data }: { entry: RecordEntry; data: RecordData } = $props();

  const id = $derived(entry.courseId ?? "");
  const info = $derived(data.lookup(id));
  const facts = $derived(data.facts(id));
  const note = $derived(gradeNote(entry.grade, info));
</script>

<CourseCardSmall
  courseId={id}
  termId={entry.term ?? null}
  title={entry.label ?? info.title ?? "Not in the catalog"}
  units={entryUnits(entry, info.units)}
  deptLabel={departmentLabel(departmentKey(id, facts))}
>
  {#snippet footer()}
    <div class="mt-2 flex items-end gap-2 border-t border-line pt-3">
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
    {#if note}<p class="mt-2 text-label-md font-normal text-muted">{note}</p>{/if}
  {/snippet}
</CourseCardSmall>
