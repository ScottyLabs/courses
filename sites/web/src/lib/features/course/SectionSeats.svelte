<script lang="ts">
  import { UsersRound } from "@lucide/svelte";
  import Popover from "$lib/components/ui/Popover.svelte";
  import { reservationCodes, reservationSummary } from "$lib/core/reservations";
  import type { Section } from "$lib/core/types";

  let { section, suffix = "" }: { section: Section; suffix?: string } = $props();

  const reservations = $derived(section.reservations ?? []);
  const count = $derived(
    section.capacity === undefined ? (reservations.length > 0 ? "Seats" : "") : `${section.capacity}${suffix}`,
  );
  const codes = $derived(reservationCodes(reservations));
</script>

{#if reservations.length === 0}
  <span class="tabular">{count}</span>
{:else}
  <Popover label="Seat reservations" placement="bottom-end" class="w-[min(90vw,20rem)] p-3">
    {#snippet trigger(props)}
      <button
        type="button"
        {...props}
        class="tabular inline-flex min-h-8 items-center gap-1 rounded-tag underline decoration-line-strong decoration-dotted underline-offset-2 hover:text-strong"
      >
        <UsersRound aria-hidden="true" class="size-3.5 text-muted" />
        {count}<span class="sr-only">, with reserved seats</span>
      </button>
    {/snippet}
    <p class="text-label-md text-muted">Seat reservations</p>
    <p class="mt-0.5 text-body-2 text-ink">Reserved seats for {reservationSummary(reservations)}</p>
    {#if codes}<p class="mt-1.5 text-label-md text-muted">{codes}</p>{/if}
    <p class="mt-1.5 text-label-md text-muted">From the Schedule of Classes.</p>
  </Popover>
{/if}
