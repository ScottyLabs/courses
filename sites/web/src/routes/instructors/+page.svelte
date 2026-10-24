<script lang="ts">
  import { CircleUser } from "@lucide/svelte";
  import PageToolbar from "$lib/components/layout/PageToolbar.svelte";
  import RailGroup from "$lib/components/layout/RailGroup.svelte";
  import RailLayout from "$lib/components/layout/RailLayout.svelte";
  import { getSession } from "$lib/components/shell/session";
  import SignInPrompt from "$lib/components/shell/SignInPrompt.svelte";
  import Title from "$lib/components/shell/Title.svelte";
  import Checkbox from "$lib/components/ui/Checkbox.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import SearchInput from "$lib/components/ui/SearchInput.svelte";
  import ViewToggle, { storedView, type ListView } from "$lib/components/ui/ViewToggle.svelte";

  const session = getSession();
  let query = $state("");
  let view = $state<ListView>(storedView("instructors") ?? "cards");
  let spring = $state(true);
  let summer = $state(false);
  let fall = $state(true);
</script>

<Title value="Instructors" />

<RailLayout storageKey="instructors" backLabel="Instructors">
  {#snippet rail()}
    <RailGroup title="Aggregate FCEs">
      <div class="flex flex-wrap justify-between gap-x-4">
        <Checkbox bind:checked={spring} label="Spring" />
        <Checkbox bind:checked={summer} label="Summer" />
        <Checkbox bind:checked={fall} label="Fall" />
      </div>
    </RailGroup>
  {/snippet}

  {#snippet header()}
    <PageToolbar title="Instructors" hideTitle="desktop">
      {#snippet search()}
        <SearchInput bind:value={query} label="Search instructors" placeholder="Search instructors by name..." />
      {/snippet}
      {#snippet actions()}
        <ViewToggle pageKey="instructors" bind:value={view} />
      {/snippet}
    </PageToolbar>
  {/snippet}

  {#if session.status === "signed-in"}
    <EmptyState
      icon={CircleUser}
      title="Instructors"
      description="Instructors and their FCE history are listed here, as cards or as a compact list."
    />
  {:else}
    <SignInPrompt
      title="Sign in to see instructors"
      description="Instructor names and FCE results are only available to signed-in CMU students."
    />
  {/if}
</RailLayout>
