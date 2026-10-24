<!--
  The rail's "Aggregate FCEs" choices, on the Courses and Instructors pages alike: how many of the most recent
  semesters with responses the stat boxes average, and in which seasons. A device convenience (fce-options.svelte.ts).
-->
<script lang="ts">
  import Checkbox from "$lib/components/ui/Checkbox.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import type { Option } from "$lib/components/ui/types";
  import type { FceSeasonLetter } from "./fce-aggregate";
  import { fceOptions } from "./fce-options.svelte";

  const id = $props.id();
  const SEASONS: { letter: FceSeasonLetter; label: string }[] = [
    { letter: "S", label: "Spring" },
    { letter: "M", label: "Summer" },
    { letter: "F", label: "Fall" },
  ];
  const COUNTS = [1, 2, 3, 4, 6];

  const options = $derived(fceOptions.value);
  // The usual counts, plus one chosen elsewhere (an older setting) so the select can show it.
  const semesterOptions = $derived.by((): Option[] => {
    const current = options.semesters;
    const counts = current === null || COUNTS.includes(current) ? COUNTS : [...COUNTS, current].toSorted((a, b) => a - b);
    return [
      ...counts.map((n) => ({ value: String(n), label: n === 1 ? "Most recent" : `Last ${n}` })),
      { value: "all", label: "All semesters" },
    ];
  });

  function toggle(letter: FceSeasonLetter, on: boolean): void {
    const seasons = on
      ? [...new Set([...options.seasons, letter])]
      : options.seasons.filter((s) => s !== letter);
    fceOptions.set({ seasons });
  }
</script>

<div class="flex items-center gap-4">
  <span aria-hidden="true" class="shrink-0 text-label-lg text-ink">Semesters to show</span>
  <Select
    label="Semesters to show"
    options={semesterOptions}
    class="min-w-0 flex-1"
    aria-describedby="{id}-help"
    bind:value={
      () => (options.semesters === null ? "all" : String(options.semesters)),
      (next) => fceOptions.set({ semesters: next === "all" ? null : Number(next) })
    }
  />
</div>
<fieldset class="-mt-2">
  <legend class="sr-only">Seasons to count</legend>
  <div class="flex flex-wrap justify-between gap-x-4 gap-y-2">
    {#each SEASONS as season (season.letter)}
      <Checkbox
        label={season.label}
        bind:checked={() => options.seasons.includes(season.letter), (on) => toggle(season.letter, on)}
      />
    {/each}
  </div>
</fieldset>
<p id="{id}-help" class="-mt-3 text-label-md font-normal text-muted">
  Workload and ratings average the most recent semesters with responses, weighted by responses.
</p>
