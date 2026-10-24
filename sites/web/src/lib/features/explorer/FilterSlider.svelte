<script lang="ts">
  let {
    label,
    help,
    range,
    anyAt,
    value,
    format,
    onchange,
  }: {
    label: string;
    /** Longer explanation, shown on hover. */
    help?: string;
    range: { min: number; max: number; step: number };
    /** The end of the track that means "any" (no filter). */
    anyAt: "min" | "max";
    value: number | null;
    format: (value: number) => string;
    onchange: (value: number | null) => void;
  } = $props();

  // A slider whose far end means "no filter". The label follows the thumb while dragging; the filter changes when
  // the thumb is released (or moved with the keyboard), so dragging doesn't rerun the search at every step.
  const id = $props.id();
  const anyValue = $derived(anyAt === "max" ? range.max : range.min);
  let draft = $state<number | null>(null);
  const shown = $derived(draft ?? value ?? anyValue);
  const isAny = $derived(shown === anyValue);
  const fill = $derived(((shown - range.min) / (range.max - range.min)) * 100);

  function read(event: Event): number {
    return Number(Number((event.currentTarget as HTMLInputElement).value).toFixed(1));
  }

  function commit(next: number): void {
    draft = null;
    const filter = next === anyValue ? null : next;
    if (filter !== value) onchange(filter);
  }
</script>

<div title={help}>
  <div class="mb-1 flex items-baseline justify-between gap-2">
    <label for={id} class="text-label-lg text-ink">{label}</label>
    <span class="tabular text-label-md {isAny ? 'text-muted' : 'text-strong'}">{isAny ? "Any" : format(shown)}</span>
  </div>
  <input
    {id}
    type="range"
    min={range.min}
    max={range.max}
    step={range.step}
    value={shown}
    aria-valuetext={isAny ? "Any" : format(shown)}
    oninput={(event) => (draft = read(event))}
    onchange={(event) => commit(read(event))}
    class="slider h-6 w-full cursor-pointer appearance-none bg-transparent"
    style="--fill:{fill}%;--from:{anyAt === 'max' ? 'var(--color-accent)' : 'var(--color-line-strong)'};--to:{anyAt ===
    'max'
      ? 'var(--color-line-strong)'
      : 'var(--color-accent)'}"
  />
  <div class="flex justify-between text-label-sm text-muted">
    <span>{anyAt === "min" ? "Any" : format(range.min)}</span>
    <span>{anyAt === "max" ? "Any" : format(range.max)}</span>
  </div>
</div>

<style>
  .slider {
    background: linear-gradient(to right, var(--from) 0 var(--fill), var(--to) var(--fill) 100%) center / 100% 2px
      no-repeat;
  }

  .slider::-webkit-slider-thumb {
    appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 999px;
    background: var(--color-accent);
    border: 2px solid var(--color-surface);
    box-shadow: 0 0 0 1px var(--color-accent);
  }

  .slider::-moz-range-thumb {
    width: 12px;
    height: 12px;
    border-radius: 999px;
    background: var(--color-accent);
    border: 2px solid var(--color-surface);
  }
</style>
