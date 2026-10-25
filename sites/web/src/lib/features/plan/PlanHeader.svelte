<script lang="ts">
  // The plan page's header: the plan picker, new plan and the plan's options (rename, duplicate, send to and import
  // from Stellic, delete), the programs audited, the plan's numbers and its overall progress.
  // Courseweave's features/degree/PlanHeader.tsx.
  import { ChevronDown, Copy, FileUp, Info, Pencil, Plus, Send, Trash2 } from "@lucide/svelte";
  import { goto } from "$app/navigation";
  import SendToStellicDialog from "$lib/components/stellic/SendToStellicDialog.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import StatBox from "$lib/components/ui/StatBox.svelte";
  import TextInput from "$lib/components/ui/TextInput.svelte";
  import { toast } from "$lib/components/ui/toast.svelte";
  import { degreeProgress, tallyRequirements } from "$lib/core/degree/progress";
  import { entryYearOf, startTermOptions } from "$lib/core/degree/hooks";
  import { termLabel } from "$lib/core/requisites";
  import { planPath } from "$lib/core/routes";
  import SioImportButton from "$lib/features/record/SioImportButton.svelte";
  import { session } from "$lib/data";
  import { personal, plans } from "$lib/state";
  import ProgramPicker from "./audit/ProgramPicker.svelte";
  import { planPage } from "./context";
  import DegreeProgressBar from "./DegreeProgressBar.svelte";
  import MenuItem from "./MenuItem.svelte";
  import OptionsMenu from "./OptionsMenu.svelte";

  const page = planPage();
  const plan = $derived(page.plan);

  let renaming = $state(false);
  let name = $state("");
  let confirmDelete = $state(false);
  let sending = $state(false);
  let importer = $state<SioImportButton>();

  const planOptions = $derived(plans.list.map((p) => ({ value: p.id, label: p.name })));
  const startOptions = $derived(
    startTermOptions(page.derived.current).map((t) => ({ value: t, label: termLabel(t) })),
  );
  const audit = $derived(page.audit);
  const requirements = $derived(
    audit?.programs.reduce((acc, p) => tallyRequirements(p.root, acc), {
      done: 0,
      inProgress: 0,
      planned: 0,
      total: 0,
    }),
  );
  const needUnits = $derived(audit?.programs[0]?.totalUnits?.need);
  const progress = $derived(degreeProgress(audit, page.derived.auditInput.courses, page.derived.unitsOf));
  // Units in the plan the degree leaves out of its total (StuCo and the like).
  const outside = $derived(
    progress && progress.unitTarget !== null
      ? page.derived.totalUnits - (progress.done + progress.inProgress + progress.planned)
      : 0,
  );

  function go(id: string): void {
    plans.setActivePlan(id);
    void goto(planPath(id), { keepFocus: true, noScroll: true });
  }

  function newPlan(): void {
    go(plans.createPlan({ startTerm: plan.startTerm, programs: plan.programs.map((p) => Object.assign({}, p)) }));
  }

</script>

<header class="flex flex-col gap-4">
  <div class="flex flex-wrap items-center gap-x-3 gap-y-3">
    <h1 class="text-heading-3 text-strong">Degree plan</h1>
    <div class="flex min-w-0 items-center gap-2 max-sm:w-full sm:ml-auto">
      <Select
        label="Plan"
        class="min-w-0 max-sm:flex-1 sm:w-64"
        value={plan.id}
        options={planOptions}
        onchange={(event) => go(event.currentTarget.value)}
      />
      <Button icon={Plus} onclick={newPlan} aria-label="New plan">
        <span class="max-sm:sr-only">New plan</span>
      </Button>
      <OptionsMenu label="Plan options" variant="outline" size="md">
        {#snippet children(close)}
          <MenuItem icon={Pencil} onclick={() => { close(); name = plan.name; renaming = true; }}>Rename</MenuItem>
          <MenuItem icon={Copy} onclick={() => { close(); go(plans.duplicatePlan(plan.id)); }}>Duplicate</MenuItem>
          <MenuItem icon={Send} onclick={() => { close(); sending = true; }}>Send to Stellic</MenuItem>
          <MenuItem icon={FileUp} onclick={() => { close(); importer?.choose(); }}>Import from Stellic</MenuItem>
          <div role="separator" class="my-1.5 h-px bg-line"></div>
          <MenuItem icon={Trash2} danger onclick={() => { close(); confirmDelete = true; }}>Delete plan</MenuItem>
        {/snippet}
      </OptionsMenu>
    </div>
  </div>

  {#if personal.mode === "draft"}
    <p class="flex items-start gap-2.5 rounded-card bg-surface px-4 py-3 text-body-2 text-ink">
      <Info aria-hidden="true" class="mt-0.5 size-4 shrink-0 text-muted" />
      <span>
        You're not signed in, so this plan is kept in this tab only.
        <button
          type="button"
          class="rounded-tag font-semibold text-accent-ink hover:underline"
          onclick={() => session.signIn()}
        >
          Sign in
        </button>
        to save it to your account and to check it against your degree requirements.
      </span>
    </p>
  {/if}

  <!-- The programs and the numbers: side by side from the width where the requirements get their own column, in
       the same columns as the semesters and the requirements below (PlanView). -->
  <div
    class="grid items-start gap-4 min-[1180px]:gap-6 {page.layout === 'table'
      ? 'min-[1180px]:grid-cols-[minmax(0,1fr)_clamp(340px,26vw,420px)]'
      : 'min-[1180px]:grid-cols-[minmax(0,1fr)_clamp(400px,31vw,500px)]'}"
  >
    <section
      aria-labelledby="plan-programs-heading"
      class="flex min-w-0 flex-col gap-3 rounded-card border border-pressed bg-surface p-4 sm:p-5"
    >
      <h2 id="plan-programs-heading" class="text-heading-6 text-strong">Programs</h2>
      <ProgramPicker
        value={plan.programs}
        onchange={(programs) => plans.setPrograms(plan.id, programs)}
        entryYearHint={entryYearOf(plan.startTerm)}
      />
    </section>

    <section
      aria-labelledby="plan-progress-heading"
      class="@container flex min-w-0 flex-col gap-4 rounded-card border border-pressed bg-surface p-4 sm:p-5"
    >
      {#if progress}
        <DegreeProgressBar headingId="plan-progress-heading" {progress} outside={outside > 0.01 ? outside : 0} />
      {:else}
        <div class="flex flex-col gap-1">
          <h2 id="plan-progress-heading" class="text-heading-6 text-strong">Degree progress</h2>
          <p class="text-body-2 text-muted">
            {plan.programs.length > 0
              ? "Checking your requirements."
              : personal.mode === "draft"
                ? "Sign in and pick a program to see what's left for your degree."
                : "Pick a program to see what's left for your degree."}
          </p>
        </div>
      {/if}
      <div class="grid grid-cols-2 gap-2 rounded-card bg-surface-2 p-2 @xl:grid-cols-4 sm:gap-3 sm:p-3">
        <StatBox
          variant="plain"
          label="Units planned"
          value={page.derived.totalUnits}
          unit={needUnits ? `of ${needUnits}` : undefined}
        />
        <div title="From your record, and AP, IB and transfer credit" class="grid">
          <StatBox variant="plain" label="Units completed" value={page.derived.completedUnits} />
        </div>
        <StatBox
          variant="plain"
          label={requirements ? "Requirements met or planned" : "Requirements"}
          value={requirements
            ? `${requirements.done + requirements.inProgress + requirements.planned}/${requirements.total}`
            : "\u2013"}
        />
        <StatBox variant="plain" label="Semesters">
          <span class="flex min-w-0 flex-wrap items-baseline gap-x-1">
            <span class="tabular text-[20px] leading-7">{page.derived.columns.length}</span>
            <label for="plan-start" class="sr-only">First semester</label>
            <span class="inline-flex items-baseline gap-x-1 whitespace-nowrap">
              <span aria-hidden="true" class="text-body-2 text-muted">from</span>
              <!-- The semester's name with a chevron right after it, over a transparent native select, so the
                   chevron stays beside the value in every browser. -->
              <span
                class="relative -my-1 inline-flex min-h-10 items-center gap-0.5 rounded-tag text-body-2 text-ink focus-ring-within md:min-h-7"
              >
                <span class="underline decoration-line-strong underline-offset-2"
                  >{startOptions.find((o) => o.value === plan.startTerm)?.label ?? termLabel(plan.startTerm)}</span
                >
                <ChevronDown aria-hidden="true" class="size-3.5 shrink-0 text-muted" />
                <select
                  id="plan-start"
                  value={plan.startTerm}
                  onchange={(event) => plans.setStartTerm(plan.id, event.currentTarget.value)}
                  class="absolute inset-0 cursor-pointer appearance-none opacity-0"
                >
                  {#each startOptions as o (o.value)}<option value={o.value}>{o.label}</option>{/each}
                </select>
              </span>
            </span>
          </span>
        </StatBox>
      </div>
    </section>
  </div>
</header>

<SioImportButton
  bind:this={importer}
  hidden
  onplancreated={go}
  onimported={(message) => {
    if (message) toast(message, { kind: "success" });
  }}
/>

<Dialog bind:open={renaming} title="Rename plan" size="sm">
  <form
    id="rename-plan"
    onsubmit={(event) => {
      event.preventDefault();
      if (name.trim() !== "") plans.renamePlan(plan.id, name.trim());
      renaming = false;
    }}
  >
    <TextInput label="Name" bind:value={name} required />
  </form>
  {#snippet footer()}
    <Button variant="ghost" onclick={() => (renaming = false)}>Cancel</Button>
    <Button variant="primary" type="submit" form="rename-plan">Save name</Button>
  {/snippet}
</Dialog>

<Dialog bind:open={confirmDelete} title="Delete plan" size="sm">
  <p class="text-body-2 text-ink">Delete {plan.name}? Your academic record isn't affected.</p>
  {#snippet footer()}
    <Button variant="ghost" onclick={() => (confirmDelete = false)}>Cancel</Button>
    <Button
      variant="danger"
      icon={Trash2}
      onclick={() => {
        const removed = plan.name;
        plans.deletePlan(plan.id);
        confirmDelete = false;
        toast(`Deleted ${removed}.`);
        void goto(planPath(plans.activeId));
      }}
    >
      Delete plan
    </Button>
  {/snippet}
</Dialog>

<SendToStellicDialog
  bind:open={sending}
  mode="plan"
  initialId={plan.id}
  courseTitle={(id) => page.lookup(id).title ?? page.lookup(id).code}
/>
