<script lang="ts">
  import { Check, Copy, Link, Share2, Trash2 } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import { toast } from "$lib/components/ui/toast.svelte";
  import { invitePath } from "$lib/core/routes";
  import { friendlyError } from "./friends";
  import { friends } from "./instance.svelte";

  // "Share Link": a single-use invite link, valid seven days. Making a new one replaces the last.
  let { open = $bindable(false) }: { open?: boolean } = $props();

  const fieldId = $props.id();
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
  let copied = $state(false);
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

  async function copy(): Promise<void> {
    try {
      await navigator.clipboard.writeText(url);
      copied = true;
      setTimeout(() => (copied = false), 2000);
    } catch {
      // Clipboard blocked: select the text so it can be copied by hand.
      const field = document.getElementById(fieldId);
      if (field instanceof HTMLInputElement) field.select();
      toast("Copying isn't allowed here. The link is selected; copy it yourself.", { kind: "info" });
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
    <label for={fieldId} class="mb-1.5 block text-label-lg text-ink">Your invite link</label>
    <div class="flex flex-col gap-2 sm:flex-row">
      <input
        id={fieldId}
        readonly
        value={url}
        aria-describedby="{fieldId}-expires"
        onfocus={(event) => event.currentTarget.select()}
        class="h-10 min-w-0 flex-1 rounded-field border border-line-strong bg-surface-2 px-4 text-body-2 text-ink focus:border-accent"
      />
      <Button variant="primary" icon={copied ? Check : Copy} onclick={copy}>{copied ? "Copied" : "Copy link"}</Button>
    </div>
    <p id="{fieldId}-expires" class="mt-2 text-label-md font-normal text-muted">
      Works once, until {expires}.
    </p>
    <div class="mt-5 flex flex-wrap gap-2">
      {#if canShare}<Button icon={Share2} onclick={share}>Share</Button>{/if}
      <Button icon={Link} loading={busy === "create"} onclick={create}>Make a new link</Button>
      <Button variant="danger" icon={Trash2} loading={busy === "revoke"} onclick={revoke}>Revoke link</Button>
    </div>
  {/if}
  {#if error}
    <p class="mt-4 text-body-2 text-danger" role="alert">{error}</p>
  {/if}
  <span class="sr-only" aria-live="polite">{copied ? "Link copied" : ""}</span>
</Dialog>
