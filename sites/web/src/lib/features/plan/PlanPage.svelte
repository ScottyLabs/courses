<script lang="ts">
  // The degree plan page: the plan named by `?plan=` (else the active one), the first-run start when there's no plan
  // yet, or "plan not found". Courseweave's features/degree/PlanPage.tsx.
  import { GraduationCap } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Spinner from "$lib/components/ui/Spinner.svelte";
  import { planPath } from "$lib/core/routes";
  import { catalog, resource, session } from "$lib/data";
  import { device, personal, plans, resolveTerm } from "$lib/state";
  import PlanView from "./PlanView.svelte";
  import StartPlan from "./StartPlan.svelte";

  let { planId, termParam }: { planId: string | null; termParam: string | null } = $props();

  const terms = resource(() => catalog.terms());
  const termId = $derived(resolveTerm(termParam, terms.value ?? [], device.lastTerm));
  const routed = resource(() => (termId === null ? null : catalog.term(termId)));

  // Signed in, wait for the account's plans so the first-run start doesn't flash before them.
  const loading = $derived(
    session.status === "loading" || (session.status === "signed-in" && personal.mode !== "account"),
  );
  const plan = $derived(planId === null ? plans.active : (plans.get(planId) ?? null));

  $effect(() => {
    if (plan !== null && plan.id !== plans.activeId) plans.setActivePlan(plan.id);
  });
</script>

{#if personal.status === "error" && personal.error !== null}
  <div role="alert" class="mx-auto mt-5 flex w-full max-w-[1600px] flex-wrap items-center gap-3 px-4 md:px-6">
    <p class="min-w-0 flex-1 rounded-card border border-danger/35 bg-danger-bg px-4 py-2.5 text-body-2 text-danger">
      {personal.error}
    </p>
    <Button size="sm" onclick={() => personal.retry()}>Retry</Button>
  </div>
{/if}

{#if loading && plan === null}
  <div class="flex justify-center py-24"><Spinner size="lg" label="Loading your plans" /></div>
{:else if planId !== null && plan === null}
  <EmptyState
    icon={GraduationCap}
    headingLevel={1}
    title="Plan not found"
    description="This plan isn't in your account. It may have been deleted on another device."
    class="py-24"
  >
    <Button variant="primary" href={planPath()}>Open your plans</Button>
  </EmptyState>
{:else if plan !== null}
  <PlanView {plan} routed={routed.value ?? null} terms={terms.value ?? []} />
{:else}
  <StartPlan />
{/if}
