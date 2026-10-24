<!-- A course's calendar color (Courseweave's ColorPicker): a swatch that opens the palette. -->
<script lang="ts">
  import { Check } from "@lucide/svelte";
  import Popover from "$lib/components/ui/Popover.svelte";
  import { COURSE_COLOR_NAMES, colorIndex, courseColorStyle } from "$lib/core/colors";

  let {
    color,
    code,
    onchange,
  }: {
    color: number;
    /** The course, for the accessible name. */
    code: string;
    onchange: (color: number) => void;
  } = $props();

  let open = $state(false);
  const current = $derived(colorIndex(color));
</script>

<Popover bind:open label="Colors for {code}" placement="bottom-start" class="p-2">
  {#snippet trigger(props)}
    <button
      type="button"
      {...props}
      aria-label="Color for {code}: {COURSE_COLOR_NAMES[current]}. Change color"
      title="Change color"
      class="grid size-10 shrink-0 place-items-center rounded-full hover:bg-hover lg:size-8"
    >
      <span
        class="course-swatch block size-4 rounded-full ring-2 ring-surface"
        style={courseColorStyle(current)}
      ></span>
    </button>
  {/snippet}
  {#snippet children({ close })}
    <div role="group" aria-label="Colors" class="grid grid-cols-5 gap-1.5">
      {#each COURSE_COLOR_NAMES as name, i (name)}
        <button
          type="button"
          aria-label={name}
          aria-pressed={i === current}
          title={name}
          onclick={() => {
            onchange(i);
            close();
          }}
          class="course-swatch grid size-9 place-items-center rounded-full text-white ring-offset-2 ring-offset-surface hover:ring-2 hover:ring-line-strong"
          style={courseColorStyle(i)}
        >
          {#if i === current}<Check aria-hidden="true" class="size-4" />{/if}
        </button>
      {/each}
    </div>
  {/snippet}
</Popover>
