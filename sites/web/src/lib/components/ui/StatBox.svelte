<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    value,
    unit,
    label,
    variant = "outline",
    class: className = "",
    children,
  }: {
    /** The figure, such as 15.19. Ignored when `children` is given. */
    value?: string | number;
    /** Smaller text after the value, such as "hrs/wk". */
    unit?: string;
    /** What the figure is, such as "Workload". */
    label: string;
    /** outline: bordered box (instructor cards); plain: white box inside a well (course card FCEs). */
    variant?: "outline" | "plain";
    class?: string;
    /** Custom value line, such as a StarRating. */
    children?: Snippet;
  } = $props();
</script>

<div
  class="flex min-w-0 flex-col justify-center rounded-tag bg-surface px-3 py-1.5 {variant === 'outline'
    ? 'border border-line-strong'
    : ''} {className}"
>
  <div class="flex items-baseline gap-1 text-ink">
    {#if children}
      {@render children()}
    {:else}
      <span class="tabular text-[20px] leading-7">{value}</span>
      {#if unit}<span class="text-body-1">{unit}</span>{/if}
    {/if}
  </div>
  <div class={variant === "outline" ? "text-body-1 text-ink" : "text-body-2 text-muted"}>{label}</div>
</div>
