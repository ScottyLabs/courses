<!--
  After signing in, asks whether what this tab made while signed out (schedules with courses, degree plans, the
  academic record, saved courses) belongs in the account, since the tab may be on a shared or lab computer. Nothing is
  saved to the account until "Add to my account"; "Discard" forgets it. Signing out instead gives the tab its drafts
  back. The root layout mounts it once; personal.heldDrafts says when to ask.
-->
<script lang="ts">
  import { Check } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import { toast } from "$lib/components/ui/toast.svelte";
  import { session } from "$lib/data";
  import { personal } from "$lib/state";

  const held = $derived(personal.heldDrafts);
  let open = $state(false);

  $effect(() => {
    if (held !== null) open = true;
  });

  const count = (n: number, one: string, many: string): string => `${n} ${n === 1 ? one : many}`;
  const items = $derived.by((): string[] => {
    if (held === null) return [];
    const list: string[] = [];
    if (held.schedules > 0) list.push(count(held.schedules, "schedule", "schedules"));
    if (held.plans > 0) list.push(count(held.plans, "degree plan", "degree plans"));
    if (held.record) list.push("courses and credit for the academic record");
    if (held.saved > 0) list.push(count(held.saved, "saved course", "saved courses"));
    return list;
  });
  const account = $derived.by(() => {
    const user = session.user;
    if (user === null) return "your account";
    return user.andrewId && user.andrewId !== user.name ? `${user.name} (${user.andrewId})` : user.name;
  });

  function add(): void {
    personal.acceptDrafts();
    open = false;
    toast("Added to your account.", { kind: "success" });
  }

  function discard(): void {
    personal.discardDrafts();
    open = false;
    toast("Discarded what this tab made while signed out.");
  }
</script>

<Dialog bind:open title="Add what you made while signed out?" size="sm" dismissible={false}>
  <div class="flex flex-col gap-3 text-body-2 text-ink">
    <p>Before you signed in, this tab made:</p>
    <ul class="flex flex-col gap-1.5">
      {#each items as item (item)}
        <li class="flex items-start gap-2">
          <Check aria-hidden="true" class="mt-0.5 size-4 shrink-0 text-muted" />{item}
        </li>
      {/each}
    </ul>
    <p>
      Add it to <span class="font-medium text-strong">{account}</span>? If someone else made it on this computer,
      discard it.
    </p>
  </div>
  {#snippet footer()}
    <Button variant="secondary" onclick={discard}>Discard</Button>
    <Button variant="primary" onclick={add}>Add to my account</Button>
  {/snippet}
</Dialog>
