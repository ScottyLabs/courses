<script lang="ts" module>
  export interface RangePreset {
    label: string;
    low: number;
    high: number;
  }
</script>

<script lang="ts">
  import { togglePill } from "./styles";

  let {
    low = $bindable(0),
    high = $bindable(24),
    min = 0,
    max = 24,
    step = 1,
    label,
    minLabel = "Minimum",
    maxLabel = "Maximum",
    format = (from: number, to: number) => `${from}-${to}`,
    presets = [],
    class: className = "",
  }: {
    low?: number;
    high?: number;
    min?: number;
    max?: number;
    step?: number;
    /** Group label, such as "Units". Each thumb is announced as "{label}: {minLabel}" and "{label}: {maxLabel}". */
    label: string;
    minLabel?: string;
    maxLabel?: string;
    /** Text shown at the end of the track for the current range. */
    format?: (low: number, high: number) => string;
    /** Quick-pick pills under the slider (Figma "6 Units ... 15 Units"). A pill is pressed when the range equals it. */
    presets?: readonly RangePreset[];
    class?: string;
  } = $props();

  const labelId = $props.id();
  const span = $derived(Math.max(max - min, 1));
  const lowPercent = $derived(((low - min) / span) * 100);
  const highPercent = $derived(((high - min) / span) * 100);
  // When both thumbs meet, the one that can still move must be on top.
  const lowOnTop = $derived(low === high && low > (min + max) / 2);

  function setLow(next: number): void {
    low = Math.min(next, high);
  }

  function setHigh(next: number): void {
    high = Math.max(next, low);
  }

  function pick(preset: RangePreset): void {
    const selected = low === preset.low && high === preset.high;
    low = selected ? min : preset.low;
    high = selected ? max : preset.high;
  }
</script>

<div role="group" aria-labelledby={labelId} class={className}>
  <span id={labelId} class="sr-only">{label}</span>
  <div class="flex items-center gap-2 pl-1">
    <div class="range relative h-6 min-w-0 flex-1">
      <div class="absolute inset-x-0 top-1/2 h-0.5 -translate-y-1/2 rounded-full bg-line-strong"></div>
      <div
        class="absolute top-1/2 h-0.5 -translate-y-1/2 rounded-full bg-accent"
        style:left="{lowPercent}%"
        style:right="{100 - highPercent}%"
      ></div>
      <input
        type="range"
        {min}
        {max}
        {step}
        bind:value={() => low, setLow}
        aria-label="{label}: {minLabel}"
        class:on-top={lowOnTop}
      />
      <input
        type="range"
        {min}
        {max}
        {step}
        bind:value={() => high, setHigh}
        aria-label="{label}: {maxLabel}"
      />
    </div>
    <output class="tabular shrink-0 text-label-lg text-ink" aria-live="polite">{format(low, high)}</output>
  </div>
  {#if presets.length > 0}
    <div class="mt-2 flex gap-1.5">
      {#each presets as preset (preset.label)}
        <button
          type="button"
          class="flex-1 whitespace-nowrap {togglePill}"
          aria-pressed={low === preset.low && high === preset.high}
          onclick={() => pick(preset)}
        >
          {preset.label}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .range input {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    margin: 0;
    appearance: none;
    background: transparent;
    pointer-events: none;
    z-index: 1;
  }

  .range input.on-top {
    z-index: 2;
  }

  .range input:focus-visible {
    outline: none;
  }

  .range input::-webkit-slider-runnable-track {
    background: transparent;
  }

  .range input::-webkit-slider-thumb {
    appearance: none;
    width: 14px;
    height: 14px;
    border: 0;
    border-radius: 999px;
    background: var(--color-accent);
    cursor: grab;
    pointer-events: auto;
    transition: box-shadow 0.15s;
  }

  .range input::-moz-range-track {
    background: transparent;
  }

  .range input::-moz-range-thumb {
    width: 14px;
    height: 14px;
    border: 0;
    border-radius: 999px;
    background: var(--color-accent);
    cursor: grab;
    pointer-events: auto;
  }

  .range input:focus-visible::-webkit-slider-thumb {
    box-shadow:
      0 0 0 2px var(--color-surface),
      0 0 0 4px var(--color-accent);
  }

  .range input:focus-visible::-moz-range-thumb {
    box-shadow:
      0 0 0 2px var(--color-surface),
      0 0 0 4px var(--color-accent);
  }

  .range input:active::-webkit-slider-thumb {
    cursor: grabbing;
  }
</style>
