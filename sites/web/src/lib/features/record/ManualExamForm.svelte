<script lang="ts">
  // Exam credit the charts don't cover (department review, college-specific mappings). Courseweave's ManualExamForm
  // in features/record/ExamSection.tsx.
  import Button from "$lib/components/ui/Button.svelte";
  import TextInput from "$lib/components/ui/TextInput.svelte";
  import { manualExamEntry, unitsError } from "$lib/core/record/forms";
  import { record } from "$lib/state";
  import CoursePicker from "./CoursePicker.svelte";
  import type { RecordData } from "./record-data.svelte";

  let { data, ondone }: { data: RecordData; ondone: () => void } = $props();

  let courseId = $state<string | null>(null);
  let name = $state("");
  let units = $state("");

  const draft = $derived(manualExamEntry({ courseId, name, units }));
  const unitsMessage = $derived(unitsError(units));

  function add(): void {
    if (draft === null) return;
    record.addEntry(draft);
    ondone();
  }
</script>

<div class="flex flex-col gap-4 border-t border-line pt-4">
  <div class="grid grid-cols-1 gap-4 md:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_96px]">
    <CoursePicker
      label="CMU course awarded"
      bind:value={courseId}
      {data}
      onpick={(id) => (units = id === null ? "" : String(data.lookup(id).units ?? ""))}
    />
    <TextInput label="Exam" bind:value={name} hint="For example: IB Further Mathematics HL, department review" />
    <TextInput label="Units" inputmode="decimal" bind:value={units} error={unitsMessage ?? undefined} />
  </div>
  <div>
    <Button variant="secondary" disabled={draft === null} onclick={add}>Add exam credit</Button>
  </div>
</div>
