<!--
  What calendar blocks show besides the course code (the Figma's "Show" dropdown; Courseweave's DisplayMenu). The
  choice is the student's calendar display preference, so every calendar on the site follows it.
-->
<script lang="ts">
  import { ChevronDown, Eye } from "@lucide/svelte";
  import Checkbox from "$lib/components/ui/Checkbox.svelte";
  import Popover from "$lib/components/ui/Popover.svelte";
  import type { CalendarDisplay } from "$lib/core/state/store";
  import { preferences } from "$lib/state";

  let open = $state(false);
  const display = $derived(preferences.calendarDisplay);

  const OPTIONS: { key: keyof CalendarDisplay; label: string; hint: string }[] = [
    { key: "section", label: "Section", hint: "Lec 1, Rec A or Sec A next to the course number" },
    { key: "time", label: "Meeting time", hint: "When the class is tall enough" },
    { key: "location", label: "Room", hint: "Building and room, when there is space" },
  ];
</script>

<Popover bind:open label="Calendar display" placement="bottom-end" class="w-72 p-2">
  {#snippet trigger(props)}
    <button
      type="button"
      {...props}
      title="Calendar display"
      class="inline-flex h-10 items-center gap-6 rounded-tag border-[0.5px] border-muted bg-surface pr-3 pl-3.5 text-body-2 text-ink hover:border-ink max-sm:w-10 max-sm:justify-center max-sm:rounded-full max-sm:border max-sm:border-line-strong max-sm:p-0 max-sm:hover:bg-hover lg:h-9 lg:w-52 lg:justify-between"
    >
      <!-- Phones show an eye icon; the name stays for screen readers. -->
      <Eye aria-hidden="true" class="size-5 sm:hidden" />
      <span class="max-sm:sr-only">Show</span>
      <ChevronDown aria-hidden="true" class="size-5 transition-transform max-sm:hidden {open ? 'rotate-180' : ''}" />
    </button>
  {/snippet}
  {#snippet children()}
    <p class="px-2 pt-1 pb-1 text-label-md font-normal text-muted">
      The course number is always shown. Choose what else each class shows on the calendar.
    </p>
    {#each OPTIONS as option (option.key)}
      <Checkbox
        checked={display[option.key]}
        label={option.label}
        description={option.hint}
        onchange={(event) =>
          preferences.setCalendarDisplay({ [option.key]: (event.currentTarget as HTMLInputElement).checked })}
        class="flex w-full rounded-tag py-1 pr-2 hover:bg-hover"
      />
    {/each}
  {/snippet}
</Popover>
