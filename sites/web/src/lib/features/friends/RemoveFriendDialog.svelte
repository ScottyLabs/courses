<script lang="ts">
  import { UserMinus } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import { toast } from "$lib/components/ui/toast.svelte";
  import type { Friend } from "$lib/api";
  import { friendlyError } from "./friends";
  import { friends } from "./instance.svelte";

  // Confirms removing a friend: access ends on both sides at once.
  let { friend = $bindable(null) }: { friend?: Friend | null } = $props();

  let removing = $state(false);
  let error = $state<string | null>(null);
  // Keeps the name on screen while the dialog closes.
  let shown = $state<Friend | null>(null);
  $effect(() => {
    if (friend !== null) {
      shown = friend;
      error = null;
    }
  });

  async function remove(): Promise<void> {
    if (shown === null) return;
    removing = true;
    error = null;
    try {
      await friends.remove(shown.id);
      toast(`Removed ${shown.name} from your friends.`, { kind: "success" });
      friend = null;
    } catch (failure) {
      error = friendlyError(failure, "They couldn't be removed. Try again.");
    } finally {
      removing = false;
    }
  }
</script>

<Dialog
  bind:open={() => friend !== null, (open) => {
    if (!open) friend = null;
  }}
  title="Remove {shown?.name ?? 'friend'}?"
  size="sm"
>
  <p class="text-body-2 text-ink">
    You'll stop seeing what {shown?.name ?? "they"} share, and they'll stop seeing what you share. They won't be notified. You
    can add each other again later.
  </p>
  {#if error}<p class="mt-3 text-body-2 text-danger" role="alert">{error}</p>{/if}
  {#snippet footer()}
    <Button variant="ghost" onclick={() => (friend = null)}>Keep friend</Button>
    <Button variant="danger" icon={UserMinus} loading={removing} onclick={remove}>Remove friend</Button>
  {/snippet}
</Dialog>
