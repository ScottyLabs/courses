<script lang="ts">
  // The degree plan's rail (decision 44): the programs audited, the plan's progress and numbers with its first
  // semester, and courses planned before their requisites. Courseweave's PlanHeader numbers and requisite problems.
  import { ChevronDown } from "@lucide/svelte";
  import { getRail } from "$lib/components/layout/rail-context";
  import RailGroup from "$lib/components/layout/RailGroup.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import StatBox from "$lib/components/ui/StatBox.svelte";
  import { entryYearOf, startTermOptions } from "$lib/core/degree/hooks";
  import { termPhase } from "$lib/core/plan";
  import { termLabel } from "$lib/core/requisites";
  import { personal, plans } from "$lib/state";
  import ProgramPicker from "./audit/ProgramPicker.svelte";
  import { planPage } from "./context";
  import DegreeProgressBar from "./DegreeProgressBar.svelte";
  import { planNumbers } from "./numbers";
  import RequisiteProblems from "./RequisiteProblems.svelte";

  const page = planPage();
  const rail = getRail();
  const plan = $derived(page.plan);
  const numbers = $derived(planNumbers(page));
  const startOptions = $derived(
    startTermOptions(page.derived.current).map((t) => ({ value: t, label: termLabel(t) })),
  );
  const problems = $derived(
    [...page.derived.checks.values()].filter(
      (c) => c.check.level === "problem" && termPhase(c.term, page.derived.current) !== "past",
    ),
  );

  function jump(planCourseId: string, term: string): void {
    // On phones the rail is a sheet over the semesters: close it first, then scroll once it's gone.
    if (rail?.asSheet === true) {
      rail.close();
      setTimeout(() => page.jumpTo(planCourseId, term), 200);
    } else page.jumpTo(planCourseId, term);
  }
</script>

<RailGroup title="Programs" count={plan.programs.length > 0 ? plan.programs.length : undefined}>
  <ProgramPicker
    value={plan.programs}
    onchange={(programs) => plans.setPrograms(plan.id, programs)}
    entryYearHint={entryYearOf(plan.startTerm)}
  />
</RailGroup>

<RailGroup title="Progress">
  {#if numbers.progress}
    <DegreeProgressBar progress={numbers.progress} outside={numbers.outside} />
  {:else}
    <p class="text-body-2 text-muted">
      {plan.programs.length > 0
        ? "Checking your requirements."
        : personal.mode === "draft"
          ? "Sign in and pick a program to see what's left for your degree."
          : "Pick a program to see what's left for your degree."}
    </p>
  {/if}
  <div class="grid grid-cols-2 gap-2 rounded-card bg-surface-2 p-2">
    <StatBox
      variant="plain"
      label="Units planned"
      value={page.derived.totalUnits}
      unit={numbers.needUnits ? `of ${numbers.needUnits}` : undefined}
    />
    <div title="From your record, and AP, IB and transfer credit" class="grid">
      <StatBox variant="plain" label="Units completed" value={page.derived.completedUnits} />
    </div>
    <StatBox
      variant="plain"
      label={numbers.requirements ? "Requirements met or planned" : "Requirements"}
      value={numbers.requirements
        ? `${numbers.requirements.done + numbers.requirements.inProgress + numbers.requirements.planned}/${numbers.requirements.total}`
        : "\u2013"}
    />
    <StatBox variant="plain" label="Semesters">
      <span class="flex min-w-0 flex-wrap items-baseline gap-x-1">
        <span class="tabular text-[20px] leading-7">{page.derived.columns.length}</span>
        <span class="inline-flex items-baseline gap-x-1 whitespace-nowrap">
          <span aria-hidden="true" class="text-body-2 text-muted">from</span>
          <!-- The semester's name with a chevron right after it. -->
          <Select
            label="First semester"
            value={plan.startTerm}
            options={startOptions}
            onchange={(event) => plans.setStartTerm(plan.id, event.currentTarget.value)}
          >
            {#snippet trigger(props)}
              <button
                {...props}
                class="-my-1 inline-flex min-h-10 items-center gap-0.5 rounded-tag text-body-2 text-ink md:min-h-7"
              >
                <span class="underline decoration-line-strong underline-offset-2"
                  >{startOptions.find((o) => o.value === plan.startTerm)?.label ?? termLabel(plan.startTerm)}</span
                >
                <ChevronDown aria-hidden="true" class="size-3.5 shrink-0 text-muted" />
              </button>
            {/snippet}
          </Select>
        </span>
      </span>
    </StatBox>
  </div>
</RailGroup>

{#if problems.length > 0}
  <RailGroup title="Requisites" count={problems.length}>
    <RequisiteProblems {problems} onjump={jump} />
  </RailGroup>
{/if}
