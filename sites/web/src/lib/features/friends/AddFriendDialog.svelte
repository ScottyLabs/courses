<script lang="ts">
  import { Link, UserPlus } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import TextInput from "$lib/components/ui/TextInput.svelte";
  import { toast } from "$lib/components/ui/toast.svelte";
  import { ApiError } from "$lib/api";
  import { friendlyError, isAndrewId, normalizeAndrewId } from "./friends";
  import { friends } from "./instance.svelte";

  // "Add Friends": a request by Andrew ID, which the other student accepts from their Friends list.
  let {
    open = $bindable(false),
    oninvite,
  }: {
    open?: boolean;
    /** Switches to the invite link, for someone who hasn't signed in yet. */
    oninvite: () => void;
  } = $props();

  const formId = $props.id();
  let andrewId = $state("");
  let error = $state<string | null>(null);
  let notSignedUp = $state(false);
  let sending = $state(false);

  $effect(() => {
    if (open) {
      andrewId = "";
      error = null;
      notSignedUp = false;
    }
  });

  async function submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (!isAndrewId(andrewId)) {
      error = "Enter an Andrew ID, such as jdoe. Letters, digits, - and _ only.";
      return;
    }
    sending = true;
    error = null;
    notSignedUp = false;
    try {
      const friend = await friends.request(andrewId);
      toast(
        friend.status === "accepted"
          ? `You and ${friend.name} are friends.`
          : `Request sent to ${friend.name}. They'll see it on their Friends list.`,
        { kind: "success" },
      );
      open = false;
    } catch (failure) {
      error = friendlyError(failure, "The request didn't go through. Try again.");
      notSignedUp =
        failure instanceof ApiError && failure.status === 400 && /signed in yet/u.test(failure.message);
    } finally {
      sending = false;
    }
  }
</script>

<Dialog
  bind:open
  title="Add a friend"
  description="Send a request by Andrew ID. You're friends once they accept."
  size="sm"
>
  <form id={formId} onsubmit={submit} novalidate>
    <TextInput
      bind:value={andrewId}
      label="Andrew ID"
      placeholder="jdoe"
      autocomplete="off"
      autocapitalize="none"
      spellcheck="false"
      required
      error={notSignedUp ? undefined : (error ?? undefined)}
      hint="They need to have signed in to CMU Courses once."
      oninput={() => (error = null)}
    />
  </form>
  {#if notSignedUp}
    <div class="mt-4 rounded-card bg-surface-2 p-3 text-body-2 text-ink" aria-live="polite">
      <p>
        {normalizeAndrewId(andrewId)} hasn't signed in yet. Send them an invite link instead; it makes you friends as soon
        as they accept it.
      </p>
      <Button size="sm" icon={Link} class="mt-3" onclick={oninvite}>Make an invite link</Button>
    </div>
  {/if}
  {#snippet footer()}
    <Button variant="ghost" onclick={() => (open = false)}>Cancel</Button>
    <Button variant="primary" type="submit" form={formId} icon={UserPlus} loading={sending}>Send request</Button>
  {/snippet}
</Dialog>
