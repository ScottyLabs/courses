<script lang="ts">
  import { LogIn } from "@lucide/svelte";
  import Button from "../ui/Button.svelte";
  import EmptyState from "../ui/EmptyState.svelte";
  import Spinner from "../ui/Spinner.svelte";
  import { getSession } from "./session";

  let {
    title = "Sign in to continue",
    description = "This page uses your CMU account. Sign in with your Andrew ID to see it.",
    class: className = "",
  }: {
    title?: string;
    description?: string;
    class?: string;
  } = $props();

  const session = getSession();
</script>

{#if session.status === "loading"}
  <div class="flex justify-center py-16 {className}"><Spinner size="lg" label="Checking your session" /></div>
{:else}
  <EmptyState icon={LogIn} {title} {description} class={className}>
    <Button variant="primary" icon={LogIn} onclick={() => session.signIn()}>Sign in</Button>
  </EmptyState>
{/if}
