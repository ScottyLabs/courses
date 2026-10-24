<!--
  Opens a schedule link with the schedule written into it (`/schedules?term=F26&import=<payload>`, Courseweave's share
  links, which work without an account): shows it and offers to save a copy. A link for another semester switches the
  page to that semester first.
-->
<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import Button from "$lib/components/ui/Button.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import { toast } from "$lib/components/ui/toast.svelte";
  import type { Term } from "$lib/core/data";
  import { decodeSchedule } from "$lib/core/share";
  import { schedules } from "$lib/state";
  import SharedSchedulePreview from "./SharedSchedulePreview.svelte";

  let { term, onswitch }: { term: Term | null; onswitch: (termId: string) => void } = $props();

  const payload = $derived(page.url.searchParams.get("import"));
  const shared = $derived(payload === null ? null : decodeSchedule(payload));
  let open = $state(false);

  $effect(() => {
    if (payload === null) return;
    if (shared !== null && term !== null && shared.termId !== term.meta.id) onswitch(shared.termId);
    else open = true;
  });

  function forget(): void {
    const url = new URL(page.url);
    url.searchParams.delete("import");
    void goto(url, { replaceState: true, keepFocus: true, noScroll: true });
  }

  function save(): void {
    if (shared === null || term === null) return;
    const taken = schedules.forTerm(term.meta.id).some((s) => s.name === shared.name);
    const entries = shared.entries.filter((e) => term.courseById.has(e.courseId));
    schedules.createSchedule(term.meta.id, taken ? `${shared.name} (shared)` : shared.name, structuredClone(entries));
    toast(`Saved "${shared.name}" to your schedules.`, { kind: "success" });
    open = false;
  }
</script>

{#if payload !== null}
  <Dialog
    bind:open
    title={shared === null ? "Couldn't open the shared schedule" : shared.name}
    description={shared === null ? undefined : "A schedule someone shared with you"}
    size="lg"
    onclose={forget}
  >
    {#if shared === null}
      <p class="text-body-2 text-ink">
        This link is incomplete or damaged. Ask whoever sent it to copy the link again.
      </p>
    {:else if term !== null && term.meta.id === shared.termId}
      <SharedSchedulePreview entries={shared.entries} {term} compact />
    {/if}
    {#snippet footer()}
      {#if shared !== null}
        <Button onclick={() => (open = false)}>Dismiss</Button>
        <Button variant="primary" disabled={term === null || term.meta.id !== shared.termId} onclick={save}
          >Save as new schedule</Button
        >
      {:else}
        <Button onclick={() => (open = false)}>Close</Button>
      {/if}
    {/snippet}
  </Dialog>
{/if}
