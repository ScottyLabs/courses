<script lang="ts">
  import { Check, X } from "@lucide/svelte";
  import Avatar from "$lib/components/ui/Avatar.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { toast } from "$lib/components/ui/toast.svelte";
  import type { Friend } from "$lib/api";
  import { friendlyError } from "./friends";
  import { friends } from "./instance.svelte";

  // Pending requests: ones to answer (accept or decline) or ones the student sent (cancel).
  let {
    kind,
    requests,
  }: {
    kind: "incoming" | "outgoing";
    requests: readonly Friend[];
  } = $props();

  const headingId = $props.id();
  let busy = $state<string | null>(null);

  async function run(id: string, action: () => Promise<void>, done: string, failed: string): Promise<void> {
    busy = id;
    try {
      await action();
      toast(done, { kind: "success" });
    } catch (error) {
      toast(friendlyError(error, failed), { kind: "error" });
    } finally {
      busy = null;
    }
  }
</script>

<section aria-labelledby={headingId}>
  <h2 id={headingId} class="text-heading-6 text-strong">
    {kind === "incoming" ? "Friend requests" : "Sent requests"}
    <span class="tabular ml-1 rounded-full bg-line px-2 text-label-md text-ink">{requests.length}</span>
  </h2>
  <p class="mt-1 text-body-2 text-muted">
    {kind === "incoming"
      ? "Accepting shares nothing by itself; you each choose what friends see."
      : "They'll see your request on their Friends list. You're friends once they accept."}
  </p>
  <ul class="mt-3 flex flex-col divide-y divide-line rounded-card border border-pressed bg-surface">
    {#each requests as request (request.id)}
      <li class="flex flex-wrap items-center gap-3 px-4 py-3">
        <Avatar name={request.name} size="sm" tone="soft" decorative />
        <div class="min-w-0 flex-1">
          <p class="truncate text-body-1 text-strong">{request.name}</p>
          <p class="truncate text-label-md font-normal text-muted">{request.andrewId}</p>
        </div>
        <div class="flex gap-2">
          {#if kind === "incoming"}
            <Button
              size="sm"
              variant="primary"
              icon={Check}
              class="max-sm:h-10"
              loading={busy === request.id}
              disabled={busy !== null}
              aria-label="Accept {request.name}'s request"
              onclick={() =>
                run(
                  request.id,
                  () => friends.accept(request.id),
                  `You and ${request.name} are friends.`,
                  "The request couldn't be accepted. Try again.",
                )}
            >
              Accept
            </Button>
            <Button
              size="sm"
              icon={X}
              class="max-sm:h-10"
              disabled={busy !== null}
              aria-label="Decline {request.name}'s request"
              onclick={() =>
                run(
                  request.id,
                  () => friends.remove(request.id),
                  `Declined ${request.name}'s request.`,
                  "The request couldn't be declined. Try again.",
                )}
            >
              Decline
            </Button>
          {:else}
            <Button
              size="sm"
              icon={X}
              class="max-sm:h-10"
              loading={busy === request.id}
              disabled={busy !== null}
              aria-label="Cancel your request to {request.name}"
              onclick={() =>
                run(
                  request.id,
                  () => friends.remove(request.id),
                  `Canceled your request to ${request.name}.`,
                  "The request couldn't be canceled. Try again.",
                )}
            >
              Cancel request
            </Button>
          {/if}
        </div>
      </li>
    {/each}
  </ul>
</section>
