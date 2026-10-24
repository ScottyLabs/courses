<script lang="ts">
  import { UserPlus } from "@lucide/svelte";
  import PageContainer from "$lib/components/layout/PageContainer.svelte";
  import { getSession } from "$lib/components/shell/session";
  import SignInPrompt from "$lib/components/shell/SignInPrompt.svelte";
  import Title from "$lib/components/shell/Title.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";

  const session = getSession();
</script>

<Title value="Friend invite" />

<PageContainer width="narrow">
  <h1 class="text-heading-3 text-strong">Friend invite</h1>
  <Card padding="lg" class="mt-6">
    {#if session.status === "signed-in"}
      <EmptyState
        size="sm"
        icon={UserPlus}
        title="Add a friend"
        description="Accepting this invite makes you friends with the student who shared the link, so you can see each other's shared courses and schedules."
      />
    {:else}
      <SignInPrompt
        title="Sign in to accept this invite"
        description="Friends are connected through CMU accounts. Sign in, then accept the invite."
      />
    {/if}
  </Card>
</PageContainer>
