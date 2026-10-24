<script lang="ts">
  import { data } from "$lib/data";
  import type { Friend, User } from "$lib/contracts";
  let { user }: { user: User | null } = $props();
  let friends = $state<Friend[]>([]);
  let andrew = $state("");
  let message = $state("");
  let invite = $state("");
  let issuedToken = $state("");
  let issuedUrl = $state("");
  let busy = $state(false);
  $effect(() => {
    if (user) void load();
    else friends = [];
    invite =
      new URLSearchParams(location.search).get("invite") ??
      sessionStorage.getItem("cmucourses-friend-invite") ??
      "";
    if (invite) sessionStorage.setItem("cmucourses-friend-invite", invite);
  });
  async function load() {
    try {
      friends = await data.friends();
    } catch (error) {
      message =
        error instanceof Error ? error.message : "Unable to load friends.";
    }
  }
  async function act(action: () => Promise<unknown>, success: string) {
    busy = true;
    try {
      await action();
      message = success;
      await load();
    } catch (error) {
      message = error instanceof Error ? error.message : "Request failed.";
    } finally {
      busy = false;
    }
  }
  async function issue() {
    const result = await data.friendInvite();
    issuedUrl = result.url;
    issuedToken = new URL(result.url).searchParams.get("invite")!;
  }
  async function redeem() {
    await data.redeemFriendInvite(invite);
    invite = "";
    sessionStorage.removeItem("cmucourses-friend-invite");
    history.replaceState({}, "", "?view=profile");
  }
</script>

{#if !user}<p><a href="/auth/login">Sign in with CMU</a> to manage friends.</p>
{:else}
  <p>
    Requests require acceptance. Shared courses appear only when both people opt
    in. Schedules are shared separately using their share links.
  </p>
  {#if invite}<button
      disabled={busy}
      onclick={() => act(redeem, "Invitation accepted.")}
      >Accept friend invitation</button
    >{/if}
  <form
    onsubmit={(event) => {
      event.preventDefault();
      void act(() => data.addFriend(andrew), "Friend request sent.");
    }}
  >
    <input
      aria-label="Andrew ID"
      bind:value={andrew}
      placeholder="Andrew ID"
      required
    /><button disabled={busy}>Send request</button>
  </form>
  <button
    disabled={busy}
    onclick={() =>
      act(
        issue,
        "Invite created. It expires in seven days; creating another replaces it.",
      )}>Create invite link</button
  >
  {#if issuedUrl}<input
      aria-label="Friend invite link"
      value={issuedUrl}
      readonly
    /><button onclick={() => navigator.clipboard.writeText(issuedUrl)}
      >Copy link</button
    ><button
      disabled={busy}
      onclick={() =>
        act(async () => {
          await data.revokeFriendInvite(issuedToken);
          issuedUrl = "";
        }, "Invite revoked.")}>Revoke invite</button
    >{/if}
  <div class="sharing">
    <button
      disabled={busy}
      onclick={() =>
        act(
          () => data.setFriendVisibility(true),
          "Shared-course matching enabled.",
        )}>Share course history with accepted friends</button
    ><button
      disabled={busy}
      onclick={() =>
        act(
          () => data.setFriendVisibility(false),
          "Course history is private.",
        )}>Keep course history private</button
    >
  </div>
  {#each friends as friend}
    <div class="friend-row">
      <strong>{friend.name}</strong><span
        >{friend.status === "incoming"
          ? "Incoming request"
          : friend.status === "outgoing"
            ? "Request pending"
            : "Friends"}</span
      >
      {#if friend.status === "incoming"}<button
          disabled={busy}
          onclick={() =>
            act(() => data.acceptFriend(friend.id), "Request accepted.")}
          >Accept</button
        >{/if}
      <button
        disabled={busy}
        onclick={() =>
          act(() => data.removeFriend(friend.id), "Relationship removed.")}
        >{friend.status === "accepted" ? "Remove" : "Cancel / decline"}</button
      >
      {#if friend.shared_courses.length}<p>
          Shared courses: {friend.shared_courses.join(", ")}
        </p>{/if}
    </div>
  {:else}<p>No friends or requests yet.</p>{/each}
{/if}
{#if message}<p role="status">{message}</p>{/if}

<style>
  form,
  .sharing,
  .friend-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.75rem;
    margin: 1rem 0;
  }
  button,
  input {
    padding: 0.5rem;
  }
</style>
