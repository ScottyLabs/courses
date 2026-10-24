<script lang="ts">
  // AP, IB and A-Level credit: pick an exam and score to see what CMU awards (the registrar's charts, by college),
  // add it, or add credit the charts don't cover by hand. Courseweave's ExamSection (features/record/ExamSection.tsx).
  import { ExternalLink, Plus, Trash2 } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import type { Option } from "$lib/components/ui/types";
  import { awardFor, EXAM_SOURCES, EXAMS, examById, type ExamBoard } from "$lib/core/exam-credit";
  import { codeOf } from "$lib/core/requisite-format";
  import { COLLEGES, type RecordEntry } from "$lib/core/state/record";
  import { record } from "$lib/state";
  import ManualExamForm from "./ManualExamForm.svelte";
  import type { RecordData } from "./record-data.svelte";
  import RecordSection from "./RecordSection.svelte";

  let { data }: { data: RecordData } = $props();

  const BOARDS: ExamBoard[] = ["AP", "IB", "A-Level"];
  const BOARD_LABEL: Record<ExamBoard, string> = { AP: "AP", IB: "IB (HL)", "A-Level": "A-Level" };

  let board = $state<ExamBoard>("AP");
  let examId = $state("");
  let score = $state("");
  let manualOpen = $state(false);

  const college = $derived(record.profile.college);
  const exams = $derived(
    EXAMS.filter((e) => e.board === board).toSorted((a, b) => a.name.localeCompare(b.name)),
  );
  const exam = $derived(examId === "" ? undefined : examById(examId));
  const examOptions = $derived<Option[]>(exams.map((e) => ({ value: e.id, label: e.name })));
  const scoreOptions = $derived<Option[]>((exam?.scores ?? []).map((s) => ({ value: s, label: s })));

  const examEntries = $derived(
    record.entries.filter((e) => e.source === "exam" || (e.source === "waiver" && e.exam !== undefined)),
  );
  const takenExams = $derived([
    ...new Set(examEntries.flatMap((e) => (e.exam === undefined ? [] : [e.exam.id]))),
  ]);
  const result = $derived(
    exam !== undefined && score !== ""
      ? awardFor(
          exam,
          score,
          college,
          takenExams.filter((id) => id !== exam.id),
        )
      : null,
  );
  const already = $derived(exam !== undefined && takenExams.includes(exam.id));
  const collegeName = $derived(COLLEGES.find((c) => c.code === college)?.name);

  function chooseBoard(next: ExamBoard): void {
    board = next;
    examId = "";
    score = "";
  }

  function add(): void {
    const award = result?.award;
    if (exam === undefined || award === null || award === undefined) return;
    record.removeExam(exam.id);
    record.addEntries(
      award.courses.map((c) => ({
        courseId: c.courseId,
        source: award.waiverOnly === true ? ("waiver" as const) : ("exam" as const),
        units: award.waiverOnly === true ? 0 : c.units,
        label: c.title,
        exam: { id: exam.id, score },
      })),
    );
    examId = "";
    score = "";
  }

  // Known exams group by exam; entries sharing a note (everything imported from SIO) group together.
  const grouped = $derived.by(() => {
    const out = new Map<string, RecordEntry[]>();
    for (const e of examEntries) {
      const key = e.exam?.id ?? (e.note === undefined ? `manual:${e.id}` : `note:${e.note}`);
      out.set(key, [...(out.get(key) ?? []), e]);
    }
    return [...out];
  });

  function rowTitle(entries: RecordEntry[]): string {
    const first = entries[0];
    const known = first?.exam === undefined ? undefined : examById(first.exam.id);
    return known ? `${known.board} ${known.name}` : (first?.note ?? "Exam credit");
  }

  function removeRow(entries: RecordEntry[]): void {
    const first = entries[0];
    if (first?.exam !== undefined && examById(first.exam.id) !== undefined) record.removeExam(first.exam.id);
    else for (const e of entries) record.removeEntry(e.id);
  }

  const warns = (note: string): boolean => note.startsWith("No ") || note.startsWith("Not ");
</script>

<RecordSection
  id="record-exams"
  title="AP, IB and A-Level credit"
  description="Pick an exam and score to see what CMU awards, from the registrar's charts (updated March 2026). Exam credit earns units but never counts toward QPA."
>
  <div class="flex flex-col gap-4 rounded-card border border-pressed bg-surface p-5">
    <div role="group" aria-label="Exam program" class="flex flex-wrap gap-2">
      {#each BOARDS as b (b)}
        <button
          type="button"
          aria-pressed={board === b}
          onclick={() => chooseBoard(b)}
          class="inline-flex h-10 items-center rounded-full border border-line-strong px-4 text-label-lg text-ink transition-colors hover:bg-hover aria-pressed:border-transparent aria-pressed:bg-line aria-pressed:text-strong"
        >
          {BOARD_LABEL[b]}
        </button>
      {/each}
    </div>
    <div class="grid grid-cols-1 gap-4 md:grid-cols-[minmax(0,1fr)_200px]">
      <Select
        label="Exam"
        hideLabel={false}
        placeholder="Choose an exam"
        options={examOptions}
        bind:value={examId}
        onchange={() => (score = "")}
      />
      <Select
        label={board === "A-Level" ? "Grade" : "Score"}
        hideLabel={false}
        placeholder={exam ? "Choose" : "Pick an exam first"}
        options={scoreOptions}
        disabled={exam === undefined}
        bind:value={score}
      />
    </div>

    {#if result}
      <div class="flex flex-col gap-2 rounded-field bg-surface-2 p-4" role="status">
        {#if result.award}
          <p class="text-body-2 text-strong">
            {result.award.waiverOnly === true ? "Places you out of" : result.eligible ? "Earns" : "Would earn"}
          </p>
          <ul class="flex flex-col gap-1">
            {#each result.award.courses as c (c.courseId)}
              <li class="flex min-w-0 items-baseline gap-2">
                <span class="tabular shrink-0 text-label-lg text-strong">{codeOf(c.courseId)}</span>
                <span class="min-w-0 flex-1 text-body-2 text-ink">{c.title}</span>
                <span class="shrink-0 text-body-2 text-muted">
                  {result.award.waiverOnly === true ? "No units" : `${c.units} units`}
                </span>
              </li>
            {/each}
          </ul>
        {/if}
        {#each result.notes as note (note)}
          <p class="text-body-2 {warns(note) ? 'text-warn' : 'text-ink'}">{note}</p>
        {/each}
        {#if result.award?.colleges && college === undefined}
          <p class="text-body-2 text-ink">Set your college under About you so this can be checked for you.</p>
        {/if}
        {#if result.award?.colleges && college !== undefined && !result.eligible}
          <p class="text-body-2 text-muted">Your college: {collegeName}.</p>
        {/if}
      </div>
    {/if}

    <div class="flex flex-wrap items-center gap-2">
      <Button variant="primary" icon={Plus} disabled={!result?.award || !result.eligible} onclick={add}>
        {already ? "Replace credit" : "Add credit"}
      </Button>
      <Button variant="ghost" aria-expanded={manualOpen} onclick={() => (manualOpen = !manualOpen)}>
        Add exam credit by hand
      </Button>
      <a
        href={EXAM_SOURCES[board].url}
        target="_blank"
        rel="noopener noreferrer"
        class="inline-flex min-h-10 items-center gap-1 text-body-2 text-accent-ink underline-offset-2 hover:underline"
      >
        Official {board} chart (PDF)
        <ExternalLink aria-hidden="true" class="size-3.5" />
        <span class="sr-only">(opens in a new tab)</span>
      </a>
    </div>
    <p class="text-label-md font-normal text-muted">{EXAM_SOURCES[board].policy}</p>
    {#if manualOpen}
      <ManualExamForm {data} ondone={() => (manualOpen = false)} />
    {/if}
  </div>

  {#if grouped.length === 0}
    <p class="rounded-card border border-dashed border-line-strong px-4 py-5 text-body-2 text-muted">
      No exam credit yet.
    </p>
  {:else}
    <ul class="divide-y divide-line overflow-hidden rounded-card border border-pressed bg-surface">
      {#each grouped as [key, entries] (key)}
        {@const first = entries[0]}
        {@const known = first?.exam === undefined ? undefined : examById(first.exam.id)}
        {@const title = rowTitle(entries)}
        {@const units = entries.reduce(
          (n, e) => n + (e.source === "waiver" ? 0 : (e.units ?? data.lookup(e.courseId ?? "").units ?? 0)),
          0,
        )}
        <li class="flex items-start gap-3 px-4 py-3">
          <div class="min-w-0 flex-1">
            <p class="text-label-lg text-strong">
              {title}{#if first?.exam && known}<span class="font-normal text-muted"
                  >, {known.board === "A-Level" ? "grade" : "score"} {first.exam.score}</span
                >{/if}
            </p>
            <p class="text-body-2 text-ink">
              {#each entries as e, i (e.id)}{i > 0 ? ", " : ""}<span class="tabular font-semibold text-strong"
                  >{data.lookup(e.courseId ?? "").code}</span
                >{e.source === "waiver" ? " (placement, no credit)" : ""}{/each}
              <span class="text-muted"> for {units} units</span>
            </p>
          </div>
          <IconButton label="Remove {title}" icon={Trash2} onclick={() => removeRow(entries)} />
        </li>
      {/each}
    </ul>
  {/if}
</RecordSection>
