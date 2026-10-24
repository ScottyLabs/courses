<script lang="ts">
  import { LockKeyhole, LogIn } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { getSession } from "$lib/components/shell/session";

  let {
    subject = "course evaluations",
    reason = "signin",
    compact = false,
    class: className = "",
  }: {
    /** What signing in shows, in the sentence "Sign in with your CMU account to see {subject}." */
    subject?: string;
    /** signin: nobody is signed in; forbidden: signed in with an account that isn't a CMU one. */
    reason?: "signin" | "forbidden";
    compact?: boolean;
    class?: string;
  } = $props();

  const session = getSession();
</script>

<div
  class="flex flex-wrap items-center gap-x-3 gap-y-2 rounded-card border border-dashed border-line-strong bg-surface-2 {compact
    ? 'px-3 py-2'
    : 'px-4 py-3'} {className}"
>
  <LockKeyhole aria-hidden="true" class="size-4 shrink-0 text-muted" />
  <p class="min-w-0 flex-1 text-ink {compact ? 'text-label-md' : 'text-body-2'}">
    {#if reason === "forbidden"}
      {subject.charAt(0).toUpperCase()}{subject.slice(1)} are only for CMU accounts. Sign in with your Andrew ID to see
      them.
    {:else}
      Sign in with your CMU account to see {subject}.
    {/if}
  </p>
  {#if session.status !== "loading"}
    <Button size="sm" variant="secondary" icon={LogIn} onclick={() => session.signIn()}>Sign in</Button>
  {/if}
</div>
