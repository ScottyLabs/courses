<script lang="ts">
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import type { Friend } from "$lib/api";
  import type { TermIndexEntry } from "$lib/core/types";
  import FriendSchedule from "./FriendSchedule.svelte";

  // A friend's shared schedules, opened from their card's "Schedule".
  let {
    open = $bindable(false),
    friend,
    terms,
    termId = $bindable(),
  }: {
    open?: boolean;
    friend: Friend | null;
    terms: readonly TermIndexEntry[];
    termId: string;
  } = $props();
</script>

<Dialog
  bind:open
  title={friend === null ? "Schedule" : `${friend.name}'s schedule`}
  description="Schedules they've made visible to friends. Only they can change them."
  size="lg"
>
  {#if friend !== null}
    <FriendSchedule {friend} {terms} bind:termId />
  {/if}
</Dialog>
