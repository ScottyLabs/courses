<script lang="ts">
  import { page } from "$app/state";
  import { CircleAlert, SearchX } from "@lucide/svelte";
  import PageContainer from "$lib/components/layout/PageContainer.svelte";
  import Title from "$lib/components/shell/Title.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";

  const notFound = $derived(page.status === 404);
</script>

<Title value={notFound ? "Page not found" : "Something went wrong"} />

<PageContainer width="narrow" class="flex flex-col justify-center">
  <EmptyState
    icon={notFound ? SearchX : CircleAlert}
    title={notFound ? "Page not found" : "Something went wrong"}
    description={notFound
      ? "There is no page at this address. It may have moved, or the link may be mistyped."
      : (page.error?.message ?? "An unexpected error occurred. Try again in a moment.")}
    headingLevel={1}
  >
    <Button variant="primary" href="/">Back to courses</Button>
    {#if !notFound}
      <Button onclick={() => location.reload()}>Reload</Button>
    {/if}
  </EmptyState>
  <p class="text-center text-label-md font-normal text-faint">Error {page.status}</p>
</PageContainer>
