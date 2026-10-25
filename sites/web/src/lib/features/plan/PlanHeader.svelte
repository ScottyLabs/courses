<script lang="ts">
  // The plan page's band above the semesters (decision 44): the title, the plan picker, new plan and the plan's
  // options (rename, duplicate, send to and import from Stellic, delete). On phones, where the rail with the
  // programs and numbers is a sheet, the button that opens it and the plan's progress in a line.
  // Courseweave's features/degree/PlanHeader.tsx.
  import { Copy, FileUp, Pencil, Plus, Send, Trash2 } from "@lucide/svelte";
  import { goto } from "$app/navigation";
  import FilterButton from "$lib/components/layout/FilterButton.svelte";
  import { getRail } from "$lib/components/layout/rail-context";
  import SendToStellicDialog from "$lib/components/stellic/SendToStellicDialog.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import TextInput from "$lib/components/ui/TextInput.svelte";
  import { toast } from "$lib/components/ui/toast.svelte";
  import { planPath } from "$lib/core/routes";
  import SioImportButton from "$lib/features/record/SioImportButton.svelte";
  import { plans } from "$lib/state";
  import { planPage } from "./context";
  import DegreeProgressBar from "./DegreeProgressBar.svelte";
  import MenuItem from "./MenuItem.svelte";
  import { planNumbers } from "./numbers";
  import OptionsMenu from "./OptionsMenu.svelte";

  const page = planPage();
  const rail = getRail();
  const plan = $derived(page.plan);
  const progress = $derived(rail?.asSheet === true ? planNumbers(page).progress : null);

  let renaming = $state(false);
  let name = $state("");
  let confirmDelete = $state(false);
  let sending = $state(false);
  let importer = $state<SioImportButton>();

  const planOptions = $derived(plans.list.map((p) => ({ value: p.id, label: p.name })));

  function go(id: string): void {
    plans.setActivePlan(id);
    void goto(planPath(id), { keepFocus: true, noScroll: true });
  }

  function newPlan(): void {
    go(plans.createPlan({ startTerm: plan.startTerm, programs: plan.programs.map((p) => Object.assign({}, p)) }));
  }
</script>

<!-- Like the profile's band: the title, with the plan's controls to its right. -->
<div class="flex flex-wrap items-center justify-between gap-x-6 gap-y-3">
  <h1 class="text-heading-4 text-strong">Degree plan</h1>
  <div class="flex min-w-0 items-center gap-2 max-sm:w-full">
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
{#if rail?.asSheet}
  <div class="mt-3 flex flex-col gap-3">
    <FilterButton class="self-start" />
    {#if progress}<DegreeProgressBar {progress} compact />{/if}
  </div>
{/if}

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
