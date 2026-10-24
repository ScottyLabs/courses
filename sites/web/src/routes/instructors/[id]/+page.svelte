<script lang="ts">
  import { page } from "$app/state";
  import { ChevronLeft, CircleUser } from "@lucide/svelte";
  import RailGroup from "$lib/components/layout/RailGroup.svelte";
  import RailLayout from "$lib/components/layout/RailLayout.svelte";
  import { getSession } from "$lib/components/shell/session";
  import SignInPrompt from "$lib/components/shell/SignInPrompt.svelte";
  import Title from "$lib/components/shell/Title.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import Checkbox from "$lib/components/ui/Checkbox.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";

  const session = getSession();
  const id = $derived(page.params.id ?? "");
  let spring = $state(true);
  let summer = $state(false);
  let fall = $state(true);
</script>

<Title value="Instructor" />

<RailLayout storageKey="instructors" backLabel="Instructor">
  {#snippet rail()}
    <RailGroup title="Aggregate FCEs">
      <div class="flex flex-wrap justify-between gap-x-4">
        <Checkbox bind:checked={spring} label="Spring" />
        <Checkbox bind:checked={summer} label="Summer" />
        <Checkbox bind:checked={fall} label="Fall" />
      </div>
    </RailGroup>
  {/snippet}

  <a
    href="/instructors"
    class="mb-4 inline-flex items-center gap-1 rounded-full py-1 pr-2 text-label-lg text-muted hover:text-ink"
  >
    <ChevronLeft aria-hidden="true" class="size-5" />All instructors
  </a>
  {#if session.status === "signed-in"}
    <Card padding="lg">
      <h1 class="sr-only">Instructor</h1>
      <EmptyState
        size="sm"
        icon={CircleUser}
        title="Instructor"
        description="This instructor's department, aggregate FCEs and course history ({id}) appear here."
      />
    </Card>
  {:else}
    <SignInPrompt
      title="Sign in to see this instructor"
      description="Instructor names and FCE results are only available to signed-in CMU students."
    />
  {/if}
</RailLayout>
