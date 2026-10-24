<script lang="ts">
  // First run: choose a first semester and programs, then create the plan, or import a Stellic plan.
  // Courseweave's StartPlan in features/degree/PlanPage.tsx.
  import { FileUp, GraduationCap } from "@lucide/svelte";
  import { goto } from "$app/navigation";
  import Button from "$lib/components/ui/Button.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import { toast } from "$lib/components/ui/toast.svelte";
  import { currentTerm, entryYearOf, startTermOptions } from "$lib/core/degree/hooks";
  import { academicYearStart, defaultTerms, isSummer } from "$lib/core/plan";
  import type { ProgramSelection } from "$lib/core/plan-types";
  import { termLabel } from "$lib/core/requisites";
  import { planPath, profilePath } from "$lib/core/routes";
  import RecordPrompt from "$lib/features/record/RecordPrompt.svelte";
  import SioImportButton from "$lib/features/record/SioImportButton.svelte";
  import { personal, plans, record } from "$lib/state";
  import ProgramPicker from "./audit/ProgramPicker.svelte";

  const current = currentTerm();
  const options = startTermOptions(current);
  const suggested = $derived.by(() => {
    const entry = record.profile.entryTerm?.toUpperCase();
    if (entry !== undefined && !isSummer(entry) && options.includes(entry)) return entry;
    return academicYearStart(current);
  });

  let chosen = $state<string | null>(null);
  const start = $derived(chosen ?? suggested);
  let programs = $state<ProgramSelection[]>([]);
  const count = $derived(record.entries.length);

  function create(): void {
    const id = plans.createPlan({ startTerm: start, programs, terms: defaultTerms(start) });
    void goto(planPath(id));
  }
</script>

<div class="mx-auto flex w-full max-w-2xl flex-col gap-6 px-4 py-8 sm:px-6 lg:py-12">
  <div>
    <h1 class="text-heading-3 text-strong">Plan your degree</h1>
    <p class="mt-2 max-w-[60ch] text-body-1 text-ink">
      Lay out every semester, see which requirements each course covers, and catch prerequisite problems before
      registration.
    </p>
  </div>

  <section aria-label="New plan" class="flex flex-col gap-6 rounded-card border border-pressed bg-surface p-5 sm:p-6">
    <div>
      <Select
        label="First semester at CMU"
        hideLabel={false}
        value={start}
        options={options.map((t) => ({ value: t, label: termLabel(t) }))}
        onchange={(event) => (chosen = event.currentTarget.value)}
      />
      <p class="mt-1.5 text-label-md font-normal text-muted">
        The plan starts with eight fall and spring semesters. You can add summers and more semesters later.
      </p>
    </div>
    <div class="flex flex-col gap-1.5">
      <h2 class="text-label-lg text-ink">Programs</h2>
      <ProgramPicker value={programs} onchange={(v) => (programs = v)} entryYearHint={entryYearOf(start)} />
    </div>
    <div class="flex flex-wrap items-center gap-3">
      <Button variant="primary" icon={GraduationCap} onclick={create}>Create plan</Button>
      <SioImportButton
        label="Import from Stellic"
        onplancreated={(id) => void goto(planPath(id))}
        onimported={(message) => {
          if (message) toast(message, { kind: "success" });
        }}
      >
        {#snippet trigger({ choose, reading })}
          <Button icon={FileUp} loading={reading} onclick={choose}>Import from Stellic</Button>
        {/snippet}
      </SioImportButton>
      <span class="text-label-md font-normal text-muted">You can change programs any time.</span>
    </div>
    <p class="-mt-3 text-label-md font-normal text-muted">
      To import, download your plan in Stellic: in Plan your Path, select Download Plan, then Create plan report. The
      file is read on this device and never uploaded.
    </p>
  </section>

  {#if count > 0}
    <p class="text-body-2 text-ink">
      Your <a href={profilePath("taken")} class="font-semibold text-accent-ink hover:underline">academic record</a> has
      {count}
      {count === 1 ? "entry" : "entries"}. Courses you've taken appear in their semesters, and AP, IB and transfer credit
      count toward requirements.
    </p>
  {:else if personal.mode !== "loading"}
    <RecordPrompt>
      Courses you've taken and AP, IB or transfer credit show up in every plan and count toward requirements.
    </RecordPrompt>
  {/if}
</div>
