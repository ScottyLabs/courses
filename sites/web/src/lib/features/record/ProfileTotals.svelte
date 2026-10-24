<script lang="ts">
  // The student's college, first semester and expected graduation, beside the record's unit totals and QPA.
  // Courseweave's ProfileAndTotals in features/record/RecordPage.tsx.
  import { ArrowRight } from "@lucide/svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import type { Option } from "$lib/components/ui/types";
  import { termLabel } from "$lib/core/requisites";
  import { planPath } from "$lib/core/routes";
  import { COLLEGES, recordTotals, type StudentProfile } from "$lib/core/state/record";
  import { record } from "$lib/state";
  import type { RecordData } from "./record-data.svelte";

  let { data }: { data: RecordData } = $props();

  const totals = $derived(recordTotals(record.entries, (id) => data.lookup(id).units));

  /** Keeps a saved value selectable even when it falls outside the generated range (an imported record). */
  function withValue(options: string[], value: string | undefined): string[] {
    return value !== undefined && value !== "" && !options.includes(value) ? [value, ...options] : options;
  }

  function termOptionList(terms: string[]): Option[] {
    return [{ value: "", label: "Not set" }, ...terms.map((t) => ({ value: t, label: termLabel(t) }))];
  }

  const collegeOptions: Option[] = [
    { value: "", label: "Not set" },
    ...COLLEGES.map((c) => ({ value: c.code, label: c.name })),
  ];
  const entryOptions = $derived(
    termOptionList(
      withValue(
        data.termChoices().filter((t) => !t.startsWith("M")),
        record.profile.entryTerm,
      ),
    ),
  );
  // Graduation options run from this year to five years out.
  const gradOptions = $derived.by(() => {
    const year = Number(data.current.slice(1));
    const out: string[] = [];
    for (let y = year; y <= year + 5; y++) {
      const yy = String(y).padStart(2, "0");
      out.push(`S${yy}`, `F${yy}`);
    }
    return termOptionList(withValue(out, record.profile.expectedGraduation));
  });

  function set(key: "college" | "entryTerm" | "expectedGraduation", value: string): void {
    const patch: Partial<StudentProfile> = { [key]: value === "" ? undefined : value };
    record.setProfile(patch);
  }

  const sources = $derived([
    { label: "CMU courses", units: totals.bySource.taken },
    { label: "Exam credit", units: totals.bySource.exam },
    { label: "Transfer credit", units: totals.bySource.transfer },
  ]);
  const fmt = (n: number): string => (Number.isInteger(n) ? String(n) : n.toFixed(1));
</script>

<div class="grid grid-cols-1 gap-4 xl:grid-cols-[minmax(0,1fr)_minmax(0,1.15fr)]">
  <section aria-labelledby="record-profile" class="rounded-card border border-pressed bg-surface p-5">
    <h2 id="record-profile" class="text-heading-5 text-strong">About you</h2>
    <div class="mt-3 grid grid-cols-1 gap-4 sm:grid-cols-2">
      <div class="sm:col-span-2">
        <Select
          label="College"
          hideLabel={false}
          options={collegeOptions}
          value={record.profile.college ?? ""}
          onchange={(event) => set("college", event.currentTarget.value)}
        />
        <p class="mt-1 text-label-md font-normal text-muted">Some exam credit depends on it.</p>
      </div>
      <Select
        label="First semester at CMU"
        hideLabel={false}
        options={entryOptions}
        value={record.profile.entryTerm ?? ""}
        onchange={(event) => set("entryTerm", event.currentTarget.value)}
      />
      <Select
        label="Expected graduation"
        hideLabel={false}
        options={gradOptions}
        value={record.profile.expectedGraduation ?? ""}
        onchange={(event) => set("expectedGraduation", event.currentTarget.value)}
      />
    </div>
  </section>

  <section aria-labelledby="record-totals" class="flex flex-col gap-3 rounded-card border border-pressed bg-surface p-5">
    <h2 id="record-totals" class="sr-only">Unit totals</h2>
    <div class="flex flex-wrap items-end gap-x-6 gap-y-2">
      <p class="flex items-baseline gap-1.5">
        <span class="tabular text-[40px] leading-none font-medium text-strong">{fmt(totals.earned)}</span>
        <span class="text-body-2 text-muted">units earned</span>
      </p>
      <p class="flex items-baseline gap-1.5">
        <span class="tabular text-heading-3 text-strong">{fmt(totals.factorable)}</span>
        <span class="text-body-2 text-muted">factorable</span>
      </p>
      <p class="flex items-baseline gap-1.5">
        <span class="tabular text-heading-3 text-strong">{fmt(totals.nonFactorable)}</span>
        <span class="text-body-2 text-muted">non-factorable</span>
      </p>
      {#if totals.qpa !== null}
        <p class="flex items-baseline gap-1.5">
          <span class="tabular text-heading-3 text-strong">{totals.qpa.toFixed(2)}</span>
          <span class="text-body-2 text-muted">QPA</span>
        </p>
      {/if}
    </div>
    <dl class="grid grid-cols-3 gap-2 border-t border-line pt-3">
      {#each sources as source (source.label)}
        <div class="min-w-0">
          <dt class="truncate text-label-md font-normal text-muted">{source.label}</dt>
          <dd class="tabular text-heading-6 text-strong">{fmt(source.units)}</dd>
        </div>
      {/each}
    </dl>
    <p class="text-label-md font-normal text-muted">
      Factorable units have letter grades (A&ndash;D, or R) and count toward QPA. Pass/fail, transfer and exam credit
      earn units without affecting QPA.{totals.ungraded > 0 ? ` ${fmt(totals.ungraded)} units have no grade yet.` : ""}
    </p>
    <a
      href={planPath()}
      class="inline-flex min-h-10 items-center gap-1 self-start text-label-lg text-accent-ink underline-offset-2 hover:underline"
    >
      See how this counts toward your degree
      <ArrowRight aria-hidden="true" class="size-4" />
    </a>
  </section>
</div>
