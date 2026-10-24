<script lang="ts">
  import { GraduationCap } from "@lucide/svelte";
  import PageToolbar from "$lib/components/layout/PageToolbar.svelte";
  import RailGroup from "$lib/components/layout/RailGroup.svelte";
  import RailLayout from "$lib/components/layout/RailLayout.svelte";
  import { getSession } from "$lib/components/shell/session";
  import SignInPrompt from "$lib/components/shell/SignInPrompt.svelte";
  import Title from "$lib/components/shell/Title.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";

  const session = getSession();
</script>

<Title value="Plan" />

<RailLayout title="Requirements" name="requirements" sheetLabel="Requirements" storageKey="plan">
  {#snippet rail()}
    <RailGroup>
      <p class="text-body-2 text-muted">
        Your programs' requirements are audited here against your plan and record.
      </p>
    </RailGroup>
  {/snippet}

  {#snippet header()}
    <PageToolbar title="Degree Plan" />
  {/snippet}

  {#if session.status === "signed-in"}
    <EmptyState
      icon={GraduationCap}
      title="Semester plan"
      description="Your semesters appear here as a grid you can drag courses between, with your progress and requisite problems."
    />
  {:else}
    <SignInPrompt
      title="Sign in to plan your degree"
      description="Degree programs come from Stellic and are only available to signed-in CMU students."
    />
  {/if}
</RailLayout>
