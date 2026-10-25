<!--
  Sharing one schedule: who among friends can see it, and a read-only link anyone can open (create, copy, turn off).
  Signed out there is no account to keep a link, so it offers Courseweave's link with the schedule written into it.
-->
<script lang="ts">
  import { Link2, LogIn, Unlink, Users } from "@lucide/svelte";
  import { api } from "$lib/api";
  import Button from "$lib/components/ui/Button.svelte";
  import CopyField from "$lib/components/ui/CopyField.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import SegmentedControl from "$lib/components/ui/SegmentedControl.svelte";
  import { toast } from "$lib/components/ui/toast.svelte";
  import { session } from "$lib/data";
  import { shareUrl } from "$lib/core/share";
  import { profilePath, sharedSchedulePath } from "$lib/core/routes";
  import type { Sharing } from "$lib/api/types";
  import { schedules, type ScheduleItem } from "$lib/state";

  let {
    open = $bindable(false),
    schedule,
    termName,
  }: { open?: boolean; schedule: ScheduleItem; termName: string } = $props();

  const signedIn = $derived(session.status === "signed-in");
  const link = $derived(
    schedule.shareId === null || schedule.shareId === undefined
      ? null
      : `${location.origin}${sharedSchedulePath(schedule.shareId)}`,
  );
  const draftLink = $derived(
    shareUrl(location.origin, {
      termId: schedule.termId,
      name: schedule.name,
      entries: schedule.entries.map((e) => ({
        courseId: e.courseId,
        section: e.section,
        child: e.child,
        color: e.color,
        hidden: e.hidden,
        passFail: e.passFail,
      })),
    }),
  );

  let busy = $state(false);
  let sharing = $state<Sharing | null>(null);

  $effect(() => {
    if (!open || !signedIn) return;
    void api.friends.sharing().then(
      (value) => (sharing = value),
      () => (sharing = null),
    );
  });

  async function createLink(): Promise<void> {
    busy = true;
    try {
      await schedules.share(schedule.id);
    } catch (error) {
      toast(`Couldn't create the link. ${error instanceof Error ? error.message : ""}`, { kind: "error" });
    } finally {
      busy = false;
    }
  }

  async function revoke(): Promise<void> {
    busy = true;
    try {
      await schedules.unshare(schedule.id);
      toast("The link no longer works.", { kind: "success" });
    } catch (error) {
      toast(`Couldn't turn off the link. ${error instanceof Error ? error.message : ""}`, { kind: "error" });
    } finally {
      busy = false;
    }
  }

  async function shareWithFriends(): Promise<void> {
    if (sharing === null) return;
    try {
      sharing = await api.friends.setSharing({ ...sharing, shareSchedules: true });
    } catch (error) {
      toast(`Couldn't change sharing. ${error instanceof Error ? error.message : ""}`, { kind: "error" });
    }
  }
</script>

<Dialog bind:open title="Share {schedule.name}" description="{termName} schedule" size="md">
  <div class="flex flex-col gap-6">
    <section class="flex flex-col gap-2" aria-labelledby="share-friends">
      <h3 id="share-friends" class="flex items-center gap-2 text-heading-6 text-strong">
        <Users aria-hidden="true" class="size-5" />Friends
      </h3>
      {#if signedIn}
        <SegmentedControl
          label="Who can see this schedule"
          value={schedule.visibility ?? "private"}
          onchange={(value) => schedules.setVisibility(schedule.id, value === "friends" ? "friends" : "private")}
          options={[
            { value: "private", label: "Only me" },
            { value: "friends", label: "Visible to friends" },
          ]}
          class="self-start"
        />
        <p class="text-body-2 text-muted">
          {schedule.visibility === "friends"
            ? "Friends you share schedules with can see it and compare it with theirs."
            : "Only you can see it."}
        </p>
        {#if schedule.visibility === "friends" && sharing !== null && !sharing.shareSchedules}
          <div class="flex flex-wrap items-center gap-3 rounded-field bg-warn-bg px-3 py-2 text-body-2 text-ink">
            <p class="min-w-0 flex-1">You don't share schedules with friends yet, so they can't see it.</p>
            <Button size="sm" variant="secondary" onclick={shareWithFriends}>Share schedules with friends</Button>
          </div>
        {/if}
        <p class="text-label-md font-normal text-muted">
          Choose friends and what they see on <a href={profilePath("friends")} class="text-accent-ink underline"
            >your profile</a
          >.
        </p>
      {:else}
        <p class="text-body-2 text-muted">Sign in to show your schedules to friends.</p>
      {/if}
    </section>

    <section class="flex flex-col gap-2 border-t border-line pt-5" aria-labelledby="share-link">
      <h3 id="share-link" class="flex items-center gap-2 text-heading-6 text-strong">
        <Link2 aria-hidden="true" class="size-5" />Link
      </h3>
      {#if signedIn}
        {#if link}
          <CopyField value={link} label="Anyone with this link can see the schedule and copy it" copyLabel="Copy link" />
          <Button variant="danger" size="sm" icon={Unlink} loading={busy} onclick={revoke} class="self-start">
            Turn off link
          </Button>
        {:else}
          <p class="text-body-2 text-muted">
            A read-only link anyone can open, without your name. You can turn it off at any time.
          </p>
          <Button
            variant="primary"
            icon={Link2}
            loading={busy}
            disabled={schedule.entries.length === 0}
            onclick={createLink}
            class="self-start"
          >
            Create link
          </Button>
          {#if schedule.entries.length === 0}
            <p class="text-label-md font-normal text-muted">Add a course first.</p>
          {/if}
        {/if}
      {:else}
        <CopyField
          value={draftLink}
          label="This link has the schedule in it: anyone who opens it can save a copy"
          copyLabel="Copy link"
        />
        <p class="text-body-2 text-muted">
          Sign in for a short link you can turn off, and to keep this schedule in your account.
        </p>
        <Button size="sm" icon={LogIn} onclick={() => session.signIn()} class="self-start">Sign in</Button>
      {/if}
    </section>
  </div>
</Dialog>
