<!--
  Compare with a friend: pick an accepted friend who shares schedules, and their schedule for this semester (one they
  marked visible to friends) shows on the calendar under yours, to find free time together.
-->
<script lang="ts">
  import { api } from "$lib/api";
  import type { Friend, FriendSchedule } from "$lib/api/types";
  import Select from "$lib/components/ui/Select.svelte";
  import { profilePath } from "$lib/core/routes";

  let {
    termId,
    termName,
    onchange,
  }: {
    termId: string;
    termName: string;
    /** The friend's schedule to show, or null for none. */
    onchange: (overlay: { owner: string; schedule: FriendSchedule } | null) => void;
  } = $props();

  let friends = $state<Friend[] | null>(null);
  let error = $state<string | null>(null);
  let friendId = $state("");
  let list = $state<FriendSchedule[] | null>(null);
  let scheduleId = $state("");
  let loading = $state(false);

  const sharing = $derived((friends ?? []).filter((f) => f.status === "accepted" && f.sharesSchedules));
  const friend = $derived(sharing.find((f) => f.id === friendId) ?? null);

  $effect(() => {
    void api.friends.list().then(
      (value) => (friends = value),
      (e: unknown) => (error = e instanceof Error ? e.message : "Couldn't load your friends."),
    );
  });

  // A friend's schedules for the semester.
  $effect(() => {
    const id = friendId;
    const term = termId;
    list = null;
    scheduleId = "";
    if (id === "") return;
    let current = true;
    loading = true;
    void api.friends.schedules(id, term).then(
      (value) => {
        if (!current) return;
        list = value.filter((s) => s.termId === term);
        scheduleId = list[0]?.id ?? "";
        loading = false;
      },
      () => {
        if (!current) return;
        list = [];
        loading = false;
      },
    );
    return () => {
      current = false;
    };
  });

  $effect(() => {
    const chosen = list?.find((s) => s.id === scheduleId) ?? null;
    onchange(friend !== null && chosen !== null ? { owner: friend.name, schedule: chosen } : null);
  });
</script>

{#if error}
  <p class="text-body-2 text-muted">{error}</p>
{:else if friends === null}
  <p class="text-body-2 text-muted" aria-live="polite">Loading friends&hellip;</p>
{:else if sharing.length === 0}
  <p class="text-body-2 text-muted">
    {friends.some((f) => f.status === "accepted")
      ? "None of your friends share their schedules yet."
      : "Add friends to compare schedules and find free time together."}
    <a href={profilePath("friends")} class="text-accent-ink underline">Friends</a>
  </p>
{:else}
  <Select
    bind:value={friendId}
    label="Show a friend's schedule"
    hideLabel={false}
    options={[{ value: "", label: "Nobody" }, ...sharing.map((f) => ({ value: f.id, label: f.name }))]}
  />
  {#if friendId !== ""}
    {#if loading}
      <p class="text-body-2 text-muted" aria-live="polite">Loading {friend?.name}'s schedules&hellip;</p>
    {:else if list !== null && list.length === 0}
      <p class="text-body-2 text-muted" aria-live="polite">
        {friend?.name} has no schedule for {termName} shared with friends.
      </p>
    {:else if list !== null && list.length > 1}
      <Select
        bind:value={scheduleId}
        label="Their schedule"
        hideLabel={false}
        options={list.map((s) => ({ value: s.id, label: s.name }))}
      />
    {/if}
    {#if list !== null && list.length > 0}
      <p class="text-label-md font-normal text-muted" aria-live="polite">
        Their classes show as striped blocks labeled with their name. Open time in both is free for both of you.
      </p>
    {/if}
  {/if}
{/if}
