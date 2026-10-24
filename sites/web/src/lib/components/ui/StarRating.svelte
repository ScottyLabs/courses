<script lang="ts">
  import { Star } from "@lucide/svelte";

  let {
    value,
    max = 5,
    size = "md",
    showValue = true,
    label = "Rating",
    class: className = "",
  }: {
    /** Rating from 0 to `max`; fractional values fill part of a star. */
    value: number;
    max?: number;
    /** sm 14px stars; md 20px stars (Figma FCE boxes). */
    size?: "sm" | "md";
    /** Show the number after the stars. */
    showValue?: boolean;
    /** What is rated, for the accessible name: "{label}: 3.91 out of 5". */
    label?: string;
    class?: string;
  } = $props();

  const fills = $derived(
    Array.from({ length: max }, (_, index) => Math.min(Math.max(value - index, 0), 1)),
  );
  const shown = $derived(Number.isFinite(value) ? value.toFixed(2).replace(/\.?0+$/u, "") : "-");
  const starSize = $derived(size === "md" ? "size-5" : "size-3.5");
</script>

<span
  role="img"
  aria-label="{label}: {shown} out of {max}"
  class="inline-flex items-center gap-1.5 {className}"
>
  <span class="inline-flex items-center gap-0.5" aria-hidden="true">
    {#each fills as fill, index (index)}
      <span class="relative {starSize}">
        <Star class="absolute inset-0 {starSize} fill-current text-pressed" strokeWidth={0} />
        <Star
          class="absolute inset-0 {starSize} fill-current text-ink"
          strokeWidth={0}
          style="clip-path: inset(0 {100 - fill * 100}% 0 0)"
        />
      </span>
    {/each}
  </span>
  {#if showValue}
    <span
      aria-hidden="true"
      class="tabular text-ink {size === 'md' ? 'text-[20px] leading-7' : 'text-label-md'}">{shown}</span
    >
  {/if}
</span>
