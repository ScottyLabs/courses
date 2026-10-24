<script lang="ts">
  import { RotateCw, Users } from "@lucide/svelte";
  import { profileBand } from "$lib/components/layout/profile-band.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
  import { api, type Friend } from "$lib/api";
  import { catalog, resource } from "$lib/data";
  import { record, schedules } from "$lib/state";
  import FriendCard from "$lib/features/friends/FriendCard.svelte";
  import FriendsActions from "$lib/features/friends/FriendsActions.svelte";
  import FriendScheduleDialog from "$lib/features/friends/FriendScheduleDialog.svelte";
  import { friendDialogs } from "$lib/features/friends/dialogs.svelte";
  import { friends } from "$lib/features/friends/instance.svelte";
  import RemoveFriendDialog from "$lib/features/friends/RemoveFriendDialog.svelte";
  import RequestList from "$lib/features/friends/RequestList.svelte";
  import SharingSettings from "$lib/features/friends/SharingSettings.svelte";
  import { currentTerm, todayInPittsburgh } from "$lib/features/friends/term";

  // Profile, Friends (new in CMU Courses; Figma "Profile Page (Friends)"): requests, friend cards with what each
  // friend shares, and the student's own sharing settings.
  let { title: _title }: { title: string } = $props();

  // "Share Link" and "Add Friends" beside the section title.
  profileBand(() => ({ actions: friendsActions }));

  $effect(() => {
    if (friends.status === "idle") void friends.load();
  });

  // "Change what friends see" in Settings links to #sharing, which appears once friends load.
  let scrolled = false;
  $effect(() => {
    if (scrolled || friends.status !== "ready" || location.hash !== "#sharing") return;
    scrolled = true;
    requestAnimationFrame(() => document.querySelector("#sharing")?.scrollIntoView({ block: "start" }));
  });

  const terms = resource(() => catalog.terms());
  let chosenTerm = $state<string | null>(null);
  const termId = $derived(
    chosenTerm ?? (terms.value ? currentTerm(terms.value, todayInPittsburgh()) : null),
  );
  const termName = $derived(terms.value?.find((t) => t.id === termId)?.name ?? "this semester");
  const term = resource(() => (termId === null ? null : catalog.term(termId)));
  const anyShares = $derived(friends.accepted.some((f) => f.sharesCourses || f.sharesSchedules));
  const taking = resource(() => (termId !== null && anyShares ? api.friends.taking(termId) : null));

  /** Course ids the student is taking that semester: their schedule shown for it and courses in progress. */
  const mine = $derived.by(() => {
    const ids = new Set<string>();
    if (termId === null) return ids;
    for (const entry of schedules.active(termId)?.entries ?? []) if (!entry.hidden) ids.add(entry.courseId);
    for (const entry of record.entries)
      if (entry.source === "taken" && entry.term === termId && (entry.grade ?? "").trim() === "" && entry.courseId)
        ids.add(entry.courseId);
    return ids;
  });

  function coursesOf(friend: Friend): string[] | null {
    if (!(friend.sharesCourses || friend.sharesSchedules)) return [];
    if (taking.value === undefined || taking.status === "loading") return null;
    return Object.entries(taking.value ?? {})
      .filter(([, names]) => names.includes(friend.name))
      .map(([id]) => id);
  }

  let scheduleOf = $state<Friend | null>(null);
  let scheduleOpen = $state(false);
  let scheduleTerm = $state("");
  let removing = $state<Friend | null>(null);

  function openSchedule(friend: Friend): void {
    scheduleOf = friend;
    scheduleTerm = termId ?? "";
    scheduleOpen = true;
  }
</script>

{#snippet friendsActions()}
  <FriendsActions />
{/snippet}

<!-- Requests and invites are answered elsewhere; coming back to the tab shows them. -->
<svelte:document
  onvisibilitychange={() => {
    if (document.visibilityState === "visible" && friends.status === "ready") void friends.load();
  }}
/>

<div class="flex flex-col gap-10">
  {#if friends.status === "error"}
    <EmptyState size="sm" icon={Users} title="Friends didn't load" description={friends.error ?? undefined}>
      <Button icon={RotateCw} onclick={() => void friends.load()}>Try again</Button>
    </EmptyState>
  {:else if friends.status !== "ready"}
    <div aria-busy="true" class="grid gap-6 sm:grid-cols-[repeat(auto-fill,minmax(280px,1fr))]">
      <span class="sr-only" aria-live="polite">Loading friends</span>
      {#each [0, 1, 2] as index (index)}<Skeleton class="h-56 rounded-card" />{/each}
    </div>
  {:else}
    {#if friends.incoming.length > 0}
      <RequestList kind="incoming" requests={friends.incoming} />
    {/if}

    <section aria-labelledby="friends-list-heading" class="flex flex-col gap-4">
      <div class="flex flex-wrap items-end justify-between gap-3">
        <h2 id="friends-list-heading" class="text-heading-6 text-strong">
          {friends.accepted.length === 0
            ? "Your friends"
            : `${friends.accepted.length} ${friends.accepted.length === 1 ? "friend" : "friends"}`}
        </h2>
        {#if terms.value && friends.accepted.length > 0}
          <Select
            label="Courses for"
            variant="compact"
            options={terms.value.map((t) => ({ value: t.id, label: t.name }))}
            value={termId ?? ""}
            onchange={(event) => (chosenTerm = event.currentTarget.value)}
            class="w-full sm:w-48 [&_select]:max-sm:h-10"
          />
        {/if}
      </div>
      {#if friends.accepted.length === 0}
        <div class="rounded-card border border-dashed border-line-strong bg-surface">
          <EmptyState
            size="sm"
            icon={Users}
            title="No friends yet"
            description="Add friends by Andrew ID, or send an invite link. You'll see the courses and schedules they choose to share."
          >
            <div class="flex flex-wrap justify-center gap-2">
              <Button variant="primary" onclick={() => (friendDialogs.adding = true)}>Add Friends</Button>
              <Button onclick={() => (friendDialogs.inviting = true)}>Share Link</Button>
            </div>
          </EmptyState>
        </div>
      {:else}
        <ul class="grid gap-6 sm:grid-cols-[repeat(auto-fill,minmax(280px,1fr))]" aria-label="Friends">
          {#each friends.accepted as friend (friend.id)}
            <li class="grid">
              <FriendCard
                {friend}
                courses={coursesOf(friend)}
                {mine}
                {termId}
                {termName}
                title={(id) => term.value?.courseById.get(id)?.title}
                onschedule={() => openSchedule(friend)}
                onremove={() => (removing = friend)}
              />
            </li>
          {/each}
        </ul>
        {#if taking.status === "error"}
          <p class="text-body-2 text-danger" role="alert">
            Friends' courses didn't load. Reload the page to try again.
          </p>
        {/if}
      {/if}
    </section>

    {#if friends.outgoing.length > 0}
      <RequestList kind="outgoing" requests={friends.outgoing} />
    {/if}

    <SharingSettings />
  {/if}
</div>

{#if terms.value}
  <FriendScheduleDialog
    bind:open={scheduleOpen}
    friend={scheduleOf}
    terms={terms.value}
    bind:termId={scheduleTerm}
  />
{/if}
<RemoveFriendDialog bind:friend={removing} />
