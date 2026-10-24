<script lang="ts">
  import { Link, Share2, Trash2 } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import CopyField from "$lib/components/ui/CopyField.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import { toast } from "$lib/components/ui/toast.svelte";
  import { invitePath } from "$lib/core/routes";
  import { friendlyError } from "./friends";
  import { friends } from "./instance.svelte";

  // "Share Link": a single-use invite link, valid seven days. Making a new one replaces the last.
  let { open = $bindable(false) }: { open?: boolean } = $props();

  const invite = $derived(friends.invite);
  // The link on this site, whatever origin the API was configured with.
  const url = $derived(invite === null ? "" : new URL(invitePath(invite.token), location.origin).href);
  const expires = $derived(
    invite === null
      ? ""
      : new Date(invite.expiresAt).toLocaleString("en-US", {
          weekday: "long",
          month: "long",
          day: "numeric",
          hour: "numeric",
          minute: "2-digit",
        }),
  );
  const canShare = typeof navigator !== "undefined" && "share" in navigator;
  let busy = $state<"create" | "revoke" | null>(null);
  let error = $state<string | null>(null);

  async function create(): Promise<void> {
    busy = "create";
    error = null;
    try {
      await friends.createInvite();
    } catch (failure) {
      error = friendlyError(failure, "The link couldn't be made. Try again.");
    } finally {
      busy = null;
    }
  }

  async function revoke(): Promise<void> {
    busy = "revoke";
    error = null;
    try {
      await friends.revokeInvite();
      toast("Invite link revoked. It no longer works.", { kind: "success" });
    } catch (failure) {
      error = friendlyError(failure, "The link couldn't be revoked. Try again.");
    } finally {
      busy = null;
    }
  }

  async function share(): Promise<void> {
    try {
      await navigator.share({ title: "Be my friend on CMU Courses", url });
    } catch {
      // Closed the share sheet.
    }
  }
</script>

<Dialog
  bind:open
  title="Invite a friend with a link"
  description="Anyone with a CMU account who opens the link can accept it, and you'll be friends right away."
  size="md"
>
  {#if invite === null}
    <ul class="flex list-disc flex-col gap-1 pl-5 text-body-2 text-ink">
      <li>The link works once and expires after seven days.</li>
      <li>Making a new link turns off the last one.</li>
      <li>Friends see only what you choose to share. You can remove a friend at any time.</li>
    </ul>
    <Button variant="primary" icon={Link} loading={busy === "create"} class="mt-5" onclick={create}>
      Make an invite link
    </Button>
  {:else}
    <CopyField value={url} label="Your invite link" copyLabel="Copy link" variant="primary" hint="Works once, until {expires}." />
    <div class="mt-5 flex flex-wrap gap-2">
      {#if canShare}<Button icon={Share2} onclick={share}>Share</Button>{/if}
      <Button icon={Link} loading={busy === "create"} onclick={create}>Make a new link</Button>
      <Button variant="danger" icon={Trash2} loading={busy === "revoke"} onclick={revoke}>Revoke link</Button>
    </div>
  {/if}
  {#if error}
    <p class="mt-4 text-body-2 text-danger" role="alert">{error}</p>
  {/if}
</Dialog>
