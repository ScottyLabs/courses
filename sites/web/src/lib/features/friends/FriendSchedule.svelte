<script lang="ts">
  import { CalendarX, EyeOff } from "@lucide/svelte";
  import Chip from "$lib/components/ui/Chip.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import Spinner from "$lib/components/ui/Spinner.svelte";
  import { api, type Friend } from "$lib/api";
  import { catalog, resource } from "$lib/data";
  import { sectionLabel } from "$lib/core/blocks";
  import { coursePath } from "$lib/core/routes";
  import { resolveEntries, unitTotals } from "$lib/core/state/schedule-view";
  import { normalizeSchedule } from "$lib/core/sync-merge";
  import { formatDays, formatLocation, formatTimeRange, formatUnits } from "$lib/core/time";
  import type { Meeting, TermIndexEntry } from "$lib/core/types";
  import { friendlyError } from "./friends";

  // A friend's schedules marked visible to friends, for one semester, as a list of courses with their sections and
  // meeting times. Mounted only while the dialog is open, so it loads then.
  let {
    friend,
    terms,
    termId = $bindable(),
  }: {
    friend: Friend;
    terms: readonly TermIndexEntry[];
    termId: string;
  } = $props();

  const fieldId = $props.id();
  const schedules = resource(() => api.friends.schedules(friend.id, termId));
  const term = resource(() => catalog.term(termId));
  const list = $derived(
    (schedules.value ?? [])
      .filter((s) => s.termId === termId)
      .map((s) => normalizeSchedule(s.id, s))
      .filter((s) => s !== null),
  );
  let chosen = $state<string | null>(null);
  const schedule = $derived(list.find((s) => s.id === chosen) ?? list[0] ?? null);
  const entries = $derived(term.value && schedule ? resolveEntries(schedule, term.value) : []);
  const visible = $derived(entries.filter((e) => !e.entry.hidden));
  const hidden = $derived(entries.filter((e) => e.entry.hidden));
  const units = $derived(unitTotals(entries));
  const termName = $derived(terms.find((t) => t.id === termId)?.name ?? termId);
  const loading = $derived(
    (schedules.status === "loading" && schedules.value === undefined) ||
      (term.status === "loading" && term.value === undefined),
  );

  function meets(meeting: Meeting): string {
    const when =
      meeting.days.length > 0 ? `${formatDays(meeting.days)} ${formatTimeRange(meeting.start, meeting.end)}` : "Time TBA";
    return `${when}, ${formatLocation(meeting)}`;
  }
</script>

<div class="flex flex-col gap-4">
  <div class="flex flex-wrap items-end gap-3">
    <Select
      label="Semester"
      hideLabel={false}
      options={terms.map((t) => ({ value: t.id, label: t.name }))}
      bind:value={termId}
      onchange={() => (chosen = null)}
      class="w-full sm:w-52"
    />
    {#if list.length > 1}
      <Select
        id={fieldId}
        label="Schedule"
        hideLabel={false}
        options={list.map((s) => ({ value: s.id, label: s.name }))}
        bind:value={() => schedule?.id ?? "", (next) => (chosen = next)}
        class="w-full sm:w-52"
      />
    {/if}
  </div>

  {#if !friend.sharesSchedules}
    <EmptyState
      size="sm"
      icon={EyeOff}
      title="{friend.name} doesn't share schedules"
      description="They can turn on Share my schedules in their friend settings."
    />
  {:else if schedules.status === "error"}
    <p class="text-body-2 text-danger" role="alert">
      {friendlyError(new Error(schedules.error ?? ""), "Their schedules didn't load. Try again in a moment.")}
    </p>
  {:else if loading}
    <div class="flex justify-center py-8"><Spinner label="Loading schedules" /></div>
  {:else if schedule === null}
    <EmptyState
      size="sm"
      icon={CalendarX}
      title="No schedule to show for {termName}"
      description="{friend.name} hasn't made a schedule for this semester visible to friends."
    />
  {:else}
    <div aria-live="polite">
      <p class="text-body-2 text-muted">
        <span class="text-ink">{schedule.name}</span>
        &middot; {units.total}{units.hasVariable ? "+" : ""} units &middot; {visible.length}
        {visible.length === 1 ? "course" : "courses"}
      </p>
      {#if visible.length === 0}
        <p class="mt-3 text-body-2 text-muted">This schedule has no courses on its calendar yet.</p>
      {:else}
        <ul class="mt-3 flex flex-col gap-2">
          {#each visible as item (item.course.id)}
            <li class="rounded-card border border-pressed bg-surface p-3">
              <div class="flex flex-wrap items-baseline gap-x-2">
                <a href={coursePath(item.course.id, termId)} class="text-heading-6 text-ink hover:text-accent-ink hover:underline"
                  >{item.course.code}</a
                >
                <span class="min-w-0 flex-1 truncate text-body-2 text-ink">{item.course.title}</span>
                <span class="text-label-md font-normal text-muted">{formatUnits(item.course.units)} units</span>
              </div>
              {#if item.parts.length === 0}
                <p class="mt-1 text-body-2 text-muted">No section chosen yet.</p>
              {/if}
              {#each item.parts as part (part.name)}
                <p class="tabular mt-1 text-body-2 text-ink">
                  <span class="inline-block w-14 font-medium">{sectionLabel(part)}</span>
                  {#each part.meetings as meeting, index (index)}{index > 0 ? "; " : ""}{meets(meeting)}{/each}
                </p>
              {/each}
              {#if item.entry.passFail}
                <Chip size="sm" tone="neutral" label="Pass/fail" class="mt-2" />
              {/if}
            </li>
          {/each}
        </ul>
      {/if}
      {#if hidden.length > 0}
        <p class="mt-3 text-label-md font-normal text-muted">
          Also in this schedule, hidden from its calendar: {hidden.map((e) => e.course.code).join(", ")}
        </p>
      {/if}
    </div>
  {/if}
</div>
