<script lang="ts">
  import { CircleCheck, Link2Off, LogIn, UserCheck, UserPlus, Users } from "@lucide/svelte";
  import PageContainer from "$lib/components/layout/PageContainer.svelte";
  import { getSession } from "$lib/components/shell/session";
  import Title from "$lib/components/shell/Title.svelte";
  import Avatar from "$lib/components/ui/Avatar.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Spinner from "$lib/components/ui/Spinner.svelte";
  import { api, ApiError, type Friend, type InviteInfo } from "$lib/api";
  import { profilePath } from "$lib/core/routes";
  import { friendlyError } from "./friends";
  import { friends } from "./instance.svelte";

  // `/invite/<token>`: who sent the invite, what accepting means, and accepting it. Signed out, the visitor signs in
  // first and comes back here.
  let { token }: { token: string } = $props();

  type State =
    | { kind: "loading" }
    | { kind: "invalid" }
    | { kind: "error"; message: string }
    | { kind: "ready"; info: InviteInfo }
    | { kind: "accepted"; friend: Friend };

  const session = getSession();
  let view = $state<State>({ kind: "loading" });
  let accepting = $state(false);
  let acceptError = $state<string | null>(null);

  $effect(() => {
    if (session.status !== "signed-in") return;
    let current = true;
    view = { kind: "loading" };
    void Promise.all([api.friends.inviteInfo(token), friends.load()]).then(
      ([info]) => {
        if (current) view = { kind: "ready", info };
      },
      (error: unknown) => {
        if (!current) return;
        view =
          error instanceof ApiError && error.status === 404
            ? { kind: "invalid" }
            : { kind: "error", message: friendlyError(error, "The invite didn't load. Try again in a moment.") };
      },
    );
    return () => {
      current = false;
    };
  });

  /** Where the two of them already stand, from the student's friends list. */
  const existing = $derived.by(() => {
    if (view.kind !== "ready") return undefined;
    const { andrewId } = view.info;
    return friends.all.find((f) => f.andrewId === andrewId);
  });

  async function accept(): Promise<void> {
    accepting = true;
    acceptError = null;
    try {
      const friend = await api.friends.redeem(token);
      friends.added(friend);
      view = { kind: "accepted", friend };
    } catch (error) {
      if (error instanceof ApiError && error.status === 404) view = { kind: "invalid" };
      else acceptError = friendlyError(error, "The invite couldn't be accepted. Try again.");
    } finally {
      accepting = false;
    }
  }

  async function acceptRequest(friend: Friend): Promise<void> {
    accepting = true;
    acceptError = null;
    try {
      await friends.accept(friend.id);
      view = { kind: "accepted", friend: { ...friend, status: "accepted" } };
    } catch (error) {
      acceptError = friendlyError(error, "The request couldn't be accepted. Try again.");
    } finally {
      accepting = false;
    }
  }

  function expiry(info: InviteInfo): string {
    return new Date(info.expiresAt).toLocaleDateString("en-US", { weekday: "long", month: "long", day: "numeric" });
  }
</script>

<Title value="Friend invite" />

{#snippet whatFriendsSee()}
  <ul class="mt-4 flex list-disc flex-col gap-1.5 pl-5 text-left text-body-2 text-ink">
    <li>You'll see what they choose to share: the courses they're taking, and schedules they make visible to friends.</li>
    <li>They see only what you choose to share. Both start off; you can change them in Friends at any time.</li>
    <li>Either of you can remove the other later.</li>
  </ul>
{/snippet}

<PageContainer width="narrow">
  <h1 class="text-heading-3 text-strong">Friend invite</h1>
  <Card padding="lg" class="mt-6" aria-live="polite">
    {#if session.status === "loading"}
      <div class="flex justify-center py-12"><Spinner size="lg" label="Checking your session" /></div>
    {:else if session.status !== "signed-in"}
      <EmptyState
        size="sm"
        icon={UserPlus}
        title="A CMU student invited you to be friends"
        description="Friends on CMU Courses can see each other's courses and schedules, when each chooses to share them. Sign in with your Andrew ID to see who sent this invite and accept it."
      >
        <Button variant="primary" icon={LogIn} onclick={() => session.signIn()}>Sign in to continue</Button>
      </EmptyState>
    {:else if view.kind === "loading"}
      <div class="flex justify-center py-12"><Spinner size="lg" label="Loading the invite" /></div>
    {:else if view.kind === "invalid"}
      <EmptyState
        size="sm"
        icon={Link2Off}
        title="This invite link doesn't work anymore"
        description="Invite links work once and expire after seven days, and making a new one turns off the last. Ask your friend for a new link, or add them by Andrew ID."
      >
        <Button href={profilePath("friends")}>Go to Friends</Button>
      </EmptyState>
    {:else if view.kind === "error"}
      <EmptyState size="sm" icon={Link2Off} title="The invite didn't load" description={view.message}>
        <Button onclick={() => location.reload()}>Try again</Button>
      </EmptyState>
    {:else if view.kind === "accepted"}
      <EmptyState
        size="sm"
        icon={CircleCheck}
        title="You and {view.friend.name} are friends"
        description="Neither of you shares anything until you choose to. Pick what your friends can see in Friends."
      >
        <div class="flex flex-wrap justify-center gap-2">
          <Button variant="primary" href="{profilePath('friends')}#sharing">Choose what you share</Button>
          <Button href={profilePath("friends")}>See your friends</Button>
        </div>
      </EmptyState>
    {:else if view.info.own}
      <EmptyState
        size="sm"
        icon={Users}
        title="This is your invite link"
        description="Send it to a friend. When they open it and accept, you'll be friends. It works once, until {expiry(view.info)}."
      >
        <Button href={profilePath("friends")}>Go to Friends</Button>
      </EmptyState>
    {:else if existing?.status === "accepted"}
      <EmptyState
        size="sm"
        icon={UserCheck}
        title="You're already friends with {existing.name}"
        description="There's nothing to accept. You can see what they share in Friends."
      >
        <Button href={profilePath("friends")}>Go to Friends</Button>
      </EmptyState>
    {:else}
      {@const info = view.info}
      <div class="flex flex-col items-center text-center">
        <Avatar name={info.name} size="lg" tone="soft" decorative />
        <h2 class="mt-4 text-heading-5 text-strong">{info.name} invited you to be friends</h2>
        <p class="text-body-2 text-muted">Andrew ID {info.andrewId}</p>
        {#if existing?.status === "incoming"}
          <p class="mt-3 text-body-2 text-ink">They've also sent you a friend request.</p>
        {:else if existing?.status === "outgoing"}
          <p class="mt-3 text-body-2 text-ink">
            You've already sent them a friend request. You'll be friends once they accept it.
          </p>
        {/if}
        {@render whatFriendsSee()}
        {#if acceptError}<p class="mt-4 text-body-2 text-danger" role="alert">{acceptError}</p>{/if}
        <div class="mt-6 flex flex-wrap justify-center gap-2">
          {#if existing?.status === "incoming"}
            <Button variant="primary" icon={UserPlus} loading={accepting} onclick={() => acceptRequest(existing)}>
              Accept
            </Button>
          {:else if existing === undefined}
            <Button variant="primary" icon={UserPlus} loading={accepting} onclick={accept}>Accept invite</Button>
          {/if}
          <Button variant="ghost" href={profilePath("friends")}>Not now</Button>
        </div>
        {#if existing === undefined}
          <p class="mt-4 text-label-md font-normal text-muted">This link works once, until {expiry(info)}.</p>
        {/if}
      </div>
    {/if}
  </Card>
</PageContainer>
