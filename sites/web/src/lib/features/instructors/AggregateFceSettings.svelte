<script lang="ts">
  import Checkbox from "$lib/components/ui/Checkbox.svelte";
  import { aggregateChoices } from "./aggregate-settings.svelte";
  import { MAX_SEMESTERS, SEASONS, type Season } from "./fce-aggregate";

  const id = $props.id();
  const labels: Record<Season, string> = { spring: "Spring", summer: "Summer", fall: "Fall" };
  const settings = $derived(aggregateChoices.value);

  function onSemesters(event: Event & { currentTarget: HTMLInputElement }): void {
    const text = event.currentTarget.value.trim();
    const count = Number.parseInt(text, 10);
    if (text === "") aggregateChoices.setSemesters(null);
    else if (Number.isInteger(count) && count >= 1) aggregateChoices.setSemesters(count);
  }
</script>

<div class="flex items-center gap-4">
  <label for="{id}-semesters" class="shrink-0 text-label-lg text-ink">Semesters to show</label>
  <input
    id="{id}-semesters"
    type="number"
    inputmode="numeric"
    min="1"
    max={MAX_SEMESTERS}
    placeholder="All"
    aria-describedby="{id}-help"
    value={settings.semesters ?? ""}
    oninput={onSemesters}
    class="h-7 min-w-0 flex-1 rounded-[8px] border border-line-strong bg-surface px-4 text-label-md font-normal text-ink transition-colors outline-none placeholder:text-muted hover:border-muted focus:border-accent"
  />
</div>
<fieldset class="-mt-2">
  <legend class="sr-only">Seasons to count</legend>
  <div class="flex flex-wrap justify-between gap-x-4 gap-y-2">
    {#each SEASONS as season (season)}
      <Checkbox
        label={labels[season]}
        checked={settings.seasons[season]}
        onchange={(event) => aggregateChoices.setSeason(season, event.currentTarget.checked)}
      />
    {/each}
  </div>
</fieldset>
<p id="{id}-help" class="-mt-3 text-label-md font-normal text-muted">
  Workload and ratings average the most recent semesters with responses{settings.semesters === null
    ? ""
    : ` (up to ${settings.semesters})`}, weighted by responses. Leave the number empty to count every semester.
</p>
