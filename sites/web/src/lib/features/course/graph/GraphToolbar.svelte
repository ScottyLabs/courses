<script lang="ts">
  // The requisite graph's settings band: what clicking a course does, whether what it unlocks shows, expanding every
  // level at once, and the zoom (out, the percentage back to 100%, in, fit). Both settings stay on this device.
  import { ChevronsDownUp, ChevronsUpDown, Maximize, Minus, Plus } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import Switch from "$lib/components/ui/Switch.svelte";
  import type { PanZoom } from "./pan-zoom.svelte";
  import { CLICK_ACTIONS, storeClickAction, storeShowUnlocks, type ClickAction } from "./settings";

  let {
    clickAction = $bindable(),
    showUnlocks = $bindable(),
    hasUnlocks,
    canExpand,
    expandedAll,
    depth,
    ontoggleall,
    panZoom,
    zoomable,
  }: {
    clickAction: ClickAction;
    showUnlocks: boolean;
    /** The course unlocks anything, so the switch has something to show. */
    hasUnlocks: boolean;
    /** Some course on the graph has requisites of its own. */
    canExpand: boolean;
    expandedAll: boolean;
    /** How many levels "Expand all" opens. */
    depth: number;
    ontoggleall: () => void;
    panZoom: PanZoom;
    /** There's more than the one course to zoom. */
    zoomable: boolean;
  } = $props();

  function chooseAction(value: string): void {
    clickAction = CLICK_ACTIONS.find((a) => a.value === value)?.value ?? "expand";
    storeClickAction(clickAction);
  }
</script>

<div class="flex flex-wrap items-center gap-x-5 gap-y-2 border-b border-line px-4 py-3">
  <div class="flex items-center gap-2">
    <span aria-hidden="true" class="text-label-lg text-ink">On click</span>
    <Select
      label="On click"
      variant="compact"
      options={CLICK_ACTIONS}
      bind:value={() => clickAction, chooseAction}
      class="w-[15rem] max-w-full"
    />
  </div>
  {#if hasUnlocks}
    <Switch
      label="Show courses it unlocks"
      labelPosition="end"
      bind:checked={showUnlocks}
      onchange={storeShowUnlocks}
      class="min-h-10"
    />
  {/if}
  {#if canExpand}
    <Button
      size="sm"
      variant="ghost"
      icon={expandedAll ? ChevronsDownUp : ChevronsUpDown}
      onclick={ontoggleall}
      class="max-md:h-10"
      title={expandedAll ? undefined : `Shows ${depth} levels of prerequisites; deeper ones keep their + button`}
    >
      {expandedAll ? "Collapse all" : "Expand all"}
    </Button>
  {/if}
  {#if zoomable}
    <div class="ml-auto flex items-center gap-0.5" role="group" aria-label="Zoom">
      <IconButton
        label="Zoom out"
        icon={Minus}
        size="sm"
        onclick={() => panZoom.set(panZoom.zoom / 1.25)}
        class="max-md:size-10"
      />
      <button
        type="button"
        onclick={() => panZoom.set(1)}
        title="Zoom to 100%"
        class="h-8 min-w-12 rounded-tag px-1 text-label-md text-ink tabular hover:bg-hover max-md:h-10"
      >
        <span class="sr-only">Zoom, </span>{Math.round(panZoom.zoom * 100)}%
      </button>
      <IconButton
        label="Zoom in"
        icon={Plus}
        size="sm"
        onclick={() => panZoom.set(panZoom.zoom * 1.25)}
        class="max-md:size-10"
      />
      <Button size="sm" variant="ghost" icon={Maximize} onclick={() => panZoom.fit()} class="max-md:h-10">Fit</Button>
    </div>
  {/if}
</div>
