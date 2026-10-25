<script lang="ts">
  // The plan page's header: the plan picker, new plan and the plan's options (rename, duplicate, send to and import
  // from Stellic, delete), the programs audited, the plan's numbers and its overall progress.
  // Courseweave's features/degree/PlanHeader.tsx.
  import { ChevronDown, Copy, FileUp, Pencil, Plus, Send, Trash2 } from "@lucide/svelte";
  import { goto } from "$app/navigation";
  import SendToStellicDialog from "$lib/components/stellic/SendToStellicDialog.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import Select from "$lib/components/ui/Select.svelte";
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
  <div class="flex flex-wrap items-center gap-x-3 gap-y-2">
    <h1 class="text-heading-3 text-strong">Degree plan</h1>
    <div class="ml-auto flex min-w-0 items-center gap-2">
      <Select
        label="Plan"
        variant="compact"
        class="max-w-[52vw] min-w-0 sm:max-w-64"
        value={plan.id}
        options={planOptions}
        onchange={(event) => go(event.currentTarget.value)}
      />
      <Button size="sm" icon={Plus} onclick={newPlan} class="max-md:h-10" aria-label="New plan">
        <span class="max-sm:sr-only">New plan</span>
      </Button>
      <OptionsMenu label="Plan options" variant="outline" class="max-md:size-10">
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
    <p class="rounded-card border border-pressed bg-surface px-4 py-2.5 text-body-2 text-ink">
      You're not signed in, so this plan is kept in this tab only.
      <button type="button" class="font-semibold text-accent-ink hover:underline" onclick={() => session.signIn()}>
        Sign in
      </button>
      to save it to your account and to check it against your degree requirements.
    </p>
  {/if}

  <div class="flex min-w-0 flex-col gap-1.5">
    <h2 class="text-label-lg text-ink">Programs</h2>
    <ProgramPicker
      value={plan.programs}
      onchange={(programs) => plans.setPrograms(plan.id, programs)}
      entryYearHint={entryYearOf(plan.startTerm)}
    />
  </div>

  <dl class="grid grid-cols-2 gap-px overflow-hidden rounded-card border border-pressed bg-line sm:grid-cols-4">
    <div class="flex flex-col bg-surface px-3 py-2.5">
      <dt class="text-label-md text-muted">Units planned</dt>
      <dd class="tabular text-heading-4 text-strong">{page.derived.totalUnits}</dd>
      {#if needUnits}<dd class="text-label-md font-normal text-muted">of {needUnits} required</dd>{/if}
    </div>
    <div class="flex flex-col bg-surface px-3 py-2.5">
      <dt class="text-label-md text-muted">Units completed</dt>
      <dd class="tabular text-heading-4 text-strong">{page.derived.completedUnits}</dd>
      <dd class="text-label-md font-normal text-muted">Record and credit</dd>
    </div>
    <div class="flex flex-col bg-surface px-3 py-2.5">
      <dt class="text-label-md text-muted">Requirements</dt>
      <dd class="tabular text-heading-4 text-strong">
        {requirements ? `${requirements.done + requirements.inProgress + requirements.planned}/${requirements.total}` : "\u2013"}
      </dd>
      <dd class="text-label-md font-normal text-muted">{requirements ? "met or planned" : "Pick a program"}</dd>
    </div>
    <div class="flex flex-col bg-surface px-3 py-2.5">
      <dt class="text-label-md text-muted">Semesters</dt>
      <dd class="tabular text-heading-4 text-strong">{page.derived.columns.length}</dd>
      <dd class="flex items-center gap-1 text-label-md font-normal text-muted">
        <label for="plan-start">Starts</label>
        <!-- The semester's name with a chevron right after it, over a transparent native select, so the chevron
             stays beside the value in every browser. -->
        <span
          class="relative -my-1 inline-flex min-h-10 items-center gap-0.5 rounded-tag pl-1 text-label-md text-ink has-[select:focus-visible]:outline-2 has-[select:focus-visible]:outline-offset-2 has-[select:focus-visible]:outline-accent md:min-h-7"
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
      </dd>
    </div>
    {#if progress}
      <div class="col-span-full bg-surface px-3 pt-2.5 pb-3">
        <DegreeProgressBar {progress} outside={outside > 0.01 ? outside : 0} />
      </div>
    {/if}
  </dl>
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
